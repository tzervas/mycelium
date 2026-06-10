//! The **v0 monomorphic typechecker** (RFC-0007 §4.4) plus the program environment it checks
//! against: the data-type registry (declarations are registry entries, never term nodes —
//! RFC-0007 §4.2) and the function table. Every refusal is an explicit [`CheckError`] — generics,
//! `spore`, value-level integers without context, and `wild` blocks (denied by default, LR-9) are
//! *refused with a reason*, never guessed at.

use std::collections::BTreeMap;

use crate::ast::{
    BaseType, Colony, Expr, FnDecl, Item, Literal, Pattern, Scalar, Strength, TypeDecl, TypeRef,
};

/// A v0 (monomorphic) type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ty {
    /// `Binary{n}`.
    Binary(u32),
    /// `Ternary{m}`.
    Ternary(u32),
    /// `Dense{d, s}`.
    Dense(u32, Scalar),
    /// A registered data type, by name (content addressing of declarations: RFC-0007 §4.2;
    /// the prototype keys by name since v0 is single-colony).
    Data(String),
    /// `Substrate{tag}` — the affine external-resource kind (LR-8). No value forms exist in v0.
    Substrate(String),
}

impl core::fmt::Display for Ty {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Ty::Binary(n) => write!(f, "Binary{{{n}}}"),
            Ty::Ternary(m) => write!(f, "Ternary{{{m}}}"),
            Ty::Dense(d, s) => write!(f, "Dense{{{d}, {s:?}}}"),
            Ty::Data(n) => write!(f, "{n}"),
            Ty::Substrate(t) => write!(f, "Substrate{{{t}}}"),
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
}

impl core::fmt::Display for CheckError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "check error in `{}`: {}", self.site, self.message)
    }
}

impl std::error::Error for CheckError {}

/// One constructor of a registered data type.
#[derive(Debug, Clone, PartialEq)]
pub struct CtorInfo {
    /// Constructor name.
    pub name: String,
    /// Field types.
    pub fields: Vec<Ty>,
}

/// A registered (monomorphic) data type.
#[derive(Debug, Clone, PartialEq)]
pub struct DataInfo {
    /// Type name.
    pub name: String,
    /// Constructors, in declaration order (the index is the `#type#i` of RFC-0007 §4.2).
    pub ctors: Vec<CtorInfo>,
}

/// The checked program environment: registry + function table. Built by [`check_colony`]; the
/// evaluator and elaborator consume it (so nothing runs unchecked).
#[derive(Debug, Clone)]
pub struct Env {
    /// Data registry, keyed by type name.
    pub types: BTreeMap<String, DataInfo>,
    /// Function table, keyed by name.
    pub fns: BTreeMap<String, FnDecl>,
    /// Per-function totality classification (RFC-0007 §4.5), filled by the totality checker.
    pub totality: BTreeMap<String, crate::totality::Totality>,
}

impl Env {
    /// Find the data type owning constructor `ctor`, with its index — `None` if no type has it.
    #[must_use]
    pub fn ctor(&self, ctor: &str) -> Option<(&DataInfo, usize)> {
        self.types
            .values()
            .find_map(|d| d.ctors.iter().position(|c| c.name == ctor).map(|i| (d, i)))
    }
}

/// The builtin prelude: `type Bool = False | True` (`if` scrutinizes it; RFC-0007 keeps `if` as
/// elaboration-level sugar over `Match` on this registry entry).
fn prelude() -> DataInfo {
    DataInfo {
        name: "Bool".to_owned(),
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

/// Resolve a surface [`TypeRef`] to a v0 [`Ty`]. Generic instantiations and VSA types are
/// explicit "deferred" refusals in v0 (RFC-0007 §4.4), never guesses. The guarantee index is
/// *allowed* and returned alongside (checked dynamically at stage 0 — RFC-0007 §4.3).
fn resolve_ty(
    site: &str,
    types: &BTreeMap<String, DataInfo>,
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
            if !args.is_empty() {
                return Err(CheckError::new(
                    site,
                    format!("generic type `{name}<…>` is deferred in v0 (RFC-0007 §4.4) — monomorphic only"),
                ));
            }
            if !types.contains_key(name) {
                return Err(CheckError::new(site, format!("unknown type `{name}`")));
            }
            Ty::Data(name.clone())
        }
    };
    Ok((base, t.guarantee))
}

/// Check a whole colony: build the registry (prelude + declarations), then type every function
/// body against its signature, classify totality, and enforce the `matured ⟹ total` gate
/// (RFC-0007 §4.5). Returns the checked [`Env`].
pub fn check_colony(colony: &Colony) -> Result<Env, CheckError> {
    let mut types = BTreeMap::new();
    let p = prelude();
    types.insert(p.name.clone(), p);

    // Pass 1: register data declarations (so they can reference each other).
    for item in &colony.items {
        if let Item::Type(td) = item {
            if !td.params.is_empty() {
                return Err(CheckError::new(
                    &td.name,
                    "generic data declarations are parsed but deferred in v0 (RFC-0007 §4.4)",
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
                    ctors: vec![],
                },
            );
        }
    }
    for item in &colony.items {
        if let Item::Type(td) = item {
            let ctors = resolve_ctors(&types, td)?;
            types.get_mut(&td.name).expect("registered above").ctors = ctors;
        }
    }

    // Pass 2: collect functions (signatures must resolve).
    let mut fns: BTreeMap<String, FnDecl> = BTreeMap::new();
    for item in &colony.items {
        match item {
            Item::Fn(fd) => {
                if !fd.sig.params.is_empty() {
                    return Err(CheckError::new(
                        &fd.sig.name,
                        "generic functions are parsed but deferred in v0 (RFC-0007 §4.4)",
                    ));
                }
                if fns.insert(fd.sig.name.clone(), fd.clone()).is_some() {
                    return Err(CheckError::new(&fd.sig.name, "duplicate function"));
                }
            }
            Item::Trait(_) | Item::Use(_) | Item::Type(_) => {}
        }
    }

    // Pass 3: type every body.
    for fd in fns.values() {
        let site = &fd.sig.name;
        let mut scope: Vec<(String, Ty)> = Vec::new();
        for p in &fd.sig.value_params {
            let (ty, _) = resolve_ty(site, &types, &p.ty)?;
            scope.push((p.name.clone(), ty));
        }
        let (ret, _) = resolve_ty(site, &types, &fd.sig.ret)?;
        let cx = Cx {
            site,
            types: &types,
            fns: &fns,
        };
        let got = cx.infer(&mut scope, &fd.body)?;
        if got != ret {
            return Err(CheckError::new(
                site,
                format!("body has type {got}, signature declares {ret}"),
            ));
        }
    }

    // Pass 4: totality classification + the matured gate (RFC-0007 §4.5).
    let totality = crate::totality::classify_all(&fns);
    for fd in fns.values() {
        if fd.matured && totality[&fd.sig.name] != crate::totality::Totality::Total {
            return Err(CheckError::new(
                &fd.sig.name,
                "`matured` requires a checked-total definition (RFC-0007 §4.5) — this one is partial",
            ));
        }
    }

    Ok(Env {
        types,
        fns,
        totality,
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
            let (ty, _) = resolve_ty(&td.name, types, f)?;
            fields.push(ty);
        }
        ctors.push(CtorInfo {
            name: c.name.clone(),
            fields,
        });
    }
    Ok(ctors)
}

/// The checking context for one function body.
struct Cx<'a> {
    site: &'a str,
    types: &'a BTreeMap<String, DataInfo>,
    fns: &'a BTreeMap<String, FnDecl>,
}

impl Cx<'_> {
    fn err<T>(&self, msg: impl Into<String>) -> Result<T, CheckError> {
        Err(CheckError::new(self.site, msg))
    }

    fn ctor(&self, name: &str) -> Option<(&DataInfo, usize)> {
        self.types
            .values()
            .find_map(|d| d.ctors.iter().position(|c| c.name == name).map(|i| (d, i)))
    }

    /// Infer the type of `e` under `scope` (a lexical stack; shadowing = later wins).
    fn infer(&self, scope: &mut Vec<(String, Ty)>, e: &Expr) -> Result<Ty, CheckError> {
        match e {
            Expr::Lit(l) => self.lit_ty(l),
            Expr::Path(p) => {
                if p.0.len() != 1 {
                    return self.err(format!(
                        "dotted path `{}` does not resolve in v0 (single-colony)",
                        p.0.join(".")
                    ));
                }
                let name = &p.0[0];
                if let Some((_, ty)) = scope.iter().rev().find(|(n, _)| n == name) {
                    return Ok(ty.clone());
                }
                if let Some((d, i)) = self.ctor(name) {
                    if d.ctors[i].fields.is_empty() {
                        return Ok(Ty::Data(d.name.clone())); // nullary ctor as a value
                    }
                    return self.err(format!(
                        "constructor `{name}` takes {} field(s) — apply it (W6 saturation)",
                        d.ctors[i].fields.len()
                    ));
                }
                self.err(format!("unknown name `{name}`"))
            }
            Expr::Let {
                name,
                ty,
                bound,
                body,
            } => {
                let bty = self.infer(scope, bound)?;
                if let Some(t) = ty {
                    let (want, _) = resolve_ty(self.site, self.types, t)?;
                    if want != bty {
                        return self.err(format!(
                            "let `{name}`: bound is {bty}, ascription says {want}"
                        ));
                    }
                }
                scope.push((name.clone(), bty));
                let r = self.infer(scope, body);
                scope.pop();
                r
            }
            Expr::If { cond, conseq, alt } => {
                let c = self.infer(scope, cond)?;
                if c != Ty::Data("Bool".to_owned()) {
                    return self.err(format!("if-condition must be Bool, got {c}"));
                }
                let t = self.infer(scope, conseq)?;
                let f = self.infer(scope, alt)?;
                if t != f {
                    return self.err(format!("if-branches disagree: {t} vs {f}"));
                }
                Ok(t)
            }
            Expr::Match { scrutinee, arms } => self.infer_match(scope, scrutinee, arms),
            Expr::Swap { value, target, .. } => {
                let vty = self.infer(scope, value)?;
                if !matches!(vty, Ty::Binary(_) | Ty::Ternary(_) | Ty::Dense(_, _)) {
                    return self.err(format!(
                        "swap source must be a representation type, got {vty}"
                    ));
                }
                let (tty, _) = resolve_ty(self.site, self.types, target)?;
                if !matches!(tty, Ty::Binary(_) | Ty::Ternary(_) | Ty::Dense(_, _)) {
                    return self.err(format!(
                        "swap target must be a representation type, got {tty}"
                    ));
                }
                Ok(tty)
            }
            Expr::Wild(_) => self.err(
                "`wild` is denied by default (LR-9): no host FFI capability exists in v0, so a \
                 wild block cannot be checked or run — this refusal is the design, not a gap",
            ),
            Expr::Spore(_) => {
                self.err("`spore` is deferred to the reconstruction-manifest work (E2-5/M-260)")
            }
            Expr::Ascribe(inner, t) => {
                let ity = self.infer(scope, inner)?;
                let (want, _) = resolve_ty(self.site, self.types, t)?;
                if ity != want {
                    return self.err(format!("ascription: expression is {ity}, ascribed {want}"));
                }
                Ok(want)
            }
            Expr::App { head, args } => self.infer_app(scope, head, args),
        }
    }

    fn infer_app(
        &self,
        scope: &mut Vec<(String, Ty)>,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Ty, CheckError> {
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
        let mut arg_tys = Vec::new();
        for a in args {
            arg_tys.push(self.infer(scope, a)?);
        }
        // User function?
        if let Some(fd) = self.fns.get(name) {
            if fd.sig.value_params.len() != arg_tys.len() {
                return self.err(format!(
                    "`{name}` takes {} argument(s), got {}",
                    fd.sig.value_params.len(),
                    arg_tys.len()
                ));
            }
            for (pm, got) in fd.sig.value_params.iter().zip(&arg_tys) {
                let (want, _) = resolve_ty(self.site, self.types, &pm.ty)?;
                if &want != got {
                    return self.err(format!(
                        "`{name}` parameter `{}` expects {want}, got {got}",
                        pm.name
                    ));
                }
            }
            let (ret, _) = resolve_ty(self.site, self.types, &fd.sig.ret)?;
            return Ok(ret);
        }
        // Constructor?
        if let Some((d, i)) = self.ctor(name) {
            let fields = &d.ctors[i].fields;
            if fields.len() != arg_tys.len() {
                return self.err(format!(
                    "constructor `{name}` takes {} field(s), got {} (W6 saturation)",
                    fields.len(),
                    arg_tys.len()
                ));
            }
            for (want, got) in fields.iter().zip(&arg_tys) {
                if want != got {
                    return self.err(format!(
                        "constructor `{name}` field expects {want}, got {got}"
                    ));
                }
            }
            return Ok(Ty::Data(d.name.clone()));
        }
        // Builtin prim?
        if let Some(ret) = prim_sig(name, &arg_tys) {
            return Ok(ret);
        }
        self.err(format!("unknown function/constructor/prim `{name}`"))
    }

    fn infer_match(
        &self,
        scope: &mut Vec<(String, Ty)>,
        scrutinee: &Expr,
        arms: &[crate::ast::Arm],
    ) -> Result<Ty, CheckError> {
        let sty = self.infer(scope, scrutinee)?;
        let Ty::Data(tname) = &sty else {
            return self.err(format!("match scrutinee must be a data type, got {sty}"));
        };
        let d = self.types.get(tname).expect("registered").clone();
        let mut covered = vec![false; d.ctors.len()];
        let mut has_default = false;
        let mut result: Option<Ty> = None;
        for arm in arms {
            if has_default {
                return self.err("arms after a wildcard/binder default are unreachable (W7)");
            }
            let pushed = match &arm.pattern {
                Pattern::Wildcard => {
                    has_default = true;
                    0
                }
                Pattern::Ident(n) => {
                    // A bare name is a nullary-ctor alternative if it names one, else a binder
                    // default (binds the scrutinee).
                    if let Some(i) = d.ctors.iter().position(|c| c.name == *n) {
                        if !d.ctors[i].fields.is_empty() {
                            return self.err(format!(
                                "constructor pattern `{n}` must bind its {} field(s) (W7)",
                                d.ctors[i].fields.len()
                            ));
                        }
                        self.mark(&mut covered, i, n)?;
                        0
                    } else {
                        has_default = true;
                        scope.push((n.clone(), sty.clone()));
                        1
                    }
                }
                Pattern::Ctor(n, subs) => {
                    let Some(i) = d.ctors.iter().position(|c| c.name == *n) else {
                        return self.err(format!("`{n}` is not a constructor of {tname}"));
                    };
                    let fields = &d.ctors[i].fields;
                    if subs.len() != fields.len() {
                        return self.err(format!(
                            "pattern `{n}` binds {} of {} field(s) (W7: exactly the arity)",
                            subs.len(),
                            fields.len()
                        ));
                    }
                    self.mark(&mut covered, i, n)?;
                    let mut pushed = 0;
                    for (sub, fty) in subs.iter().zip(fields) {
                        match sub {
                            Pattern::Ident(b) if self.ctor(b).is_none() => {
                                scope.push((b.clone(), fty.clone()));
                                pushed += 1;
                            }
                            Pattern::Wildcard => {}
                            _ => {
                                return self.err(
                                    "nested patterns are an L2 concern; W7 match is flat — bind \
                                     fields to names and match again",
                                )
                            }
                        }
                    }
                    pushed
                }
                Pattern::Lit(_) => {
                    return self.err("literal patterns are deferred in v0 (match on data types)")
                }
            };
            let bty = self.infer(scope, &arm.body)?;
            for _ in 0..pushed {
                scope.pop();
            }
            match &result {
                None => result = Some(bty),
                Some(r) if *r != bty => {
                    return self.err(format!("match arms disagree: {r} vs {bty}"))
                }
                Some(_) => {}
            }
        }
        if !has_default && covered.iter().any(|c| !c) {
            let missing: Vec<&str> = d
                .ctors
                .iter()
                .zip(&covered)
                .filter(|(_, c)| !**c)
                .map(|(c, _)| c.name.as_str())
                .collect();
            return self.err(format!(
                "non-exhaustive match on {tname}: missing {} (W7 — coverage is checked, never assumed)",
                missing.join(", ")
            ));
        }
        result.map_or_else(|| self.err("a match needs at least one arm"), Ok)
    }

    fn mark(&self, covered: &mut [bool], i: usize, name: &str) -> Result<(), CheckError> {
        if covered[i] {
            return self.err(format!(
                "duplicate alternative for constructor `{name}` (W7)"
            ));
        }
        covered[i] = true;
        Ok(())
    }

    /// Literal typing (Q6): a literal *is* its representation — a binary literal's width is its
    /// digit count, a ternary literal's trit count its width. Bare integers and lists need
    /// context v0 does not yet give them → explicit refusal, never a cross-family default.
    fn lit_ty(&self, l: &Literal) -> Result<Ty, CheckError> {
        match l {
            Literal::Bin(s) => {
                let n = s.chars().filter(|c| *c == '0' || *c == '1').count();
                if n == 0 {
                    return self.err("empty binary literal");
                }
                Ok(Ty::Binary(u32::try_from(n).expect("digit count fits u32")))
            }
            Literal::Trit(s) => {
                if s.is_empty() {
                    return self.err("empty ternary literal");
                }
                Ok(Ty::Ternary(
                    u32::try_from(s.len()).expect("trit count fits u32"),
                ))
            }
            Literal::Int(_) => self.err(
                "a bare integer literal has no representation family (no cross-family defaulting, \
                 Q6) — write a binary/ternary literal or an ascribed Dense element",
            ),
            Literal::List(_) => self.err("list literals are deferred in v0 (Dense construction)"),
        }
    }
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
