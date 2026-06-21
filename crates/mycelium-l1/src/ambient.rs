//! **Ambient representation resolution** (RFC-0012; enacts M-344). The ambient is a *declared,
//! scoped, paradigm-only* default that offsets honesty's verbosity (tension **A**) **without**
//! becoming a black box. This module is the enactment's honest core.
//!
//! # The architecture: a surface→surface "expand to longhand" pass
//! [`resolve`] rewrites a parsed [`Nodule`] into its **longhand twin**: every paradigm-less repr
//! `{…}` is replaced by the concrete `P{…}` of the enclosing ambient, every `with paradigm` block
//! is stripped (after filling its interior tags), and every bare decimal under an ambient is tagged
//! [`Literal::AmbientInt`] (its *width* is resolved later, by the checker, from context — §4.3).
//! The **unchanged** `check_nodule → elaborate` pipeline then runs on the twin. This makes
//! RFC-0012's two normative invariants true *by construction*:
//!
//! - **(I1) the ambient emits no `Swap`** — resolution only fills tags/encodings; it has no rule
//!   that inserts a [`Expr::Swap`] (conversions stay author-written, WF1);
//! - **(I2) resolution is observationally the identity** — `elaborate(p) = elaborate(resolve(p))`,
//!   and `resolve(p)` *is* the longhand twin a reader would write, so the content hashes coincide
//!   (RFC-0001 §4.6). The §4.6 meaning-preservation differential (`tests/ambient.rs`) is the
//!   executable proof.
//!
//! # Never-silent refusals (§4.3/§4.4), the no-black-box guarantee
//! - [`AmbientError::UnresolvedAmbient`] — a `{…}` with no enclosing ambient (no implicit global
//!   fallback — that would be silent).
//! - [`AmbientError::ParadigmShapeMismatch`] — a written shape that does not fit the ambient
//!   paradigm (e.g. `{8}` under a `Dense` ambient) — never a coerced guess.
//! - [`AmbientError::BareDecimalNoEncoding`] — a bare decimal under a `Dense`/`VSA` ambient (those
//!   paradigms have no bare-decimal encoding).
//! - [`AmbientError::MultipleDefaults`] — two nodule-scope `default paradigm` declarations.
//!
//! The cross-paradigm `MissingConversion` refusal (§4.4) and the bare-decimal `UnresolvedWidth`
//! refusal (§4.3) need *types*, so they live in the checker ([`crate::checkty`]); they are the
//! never-silent edge-of-the-feature guarantees.
//!
//! # Provenance / EXPLAIN (§4.3), the no-black-box guarantee #2
//! [`resolve_report`] returns, alongside the twin, a [`ResolutionNote`] per fill — *where did this
//! paradigm come from?* answered for every resolved site. The resolved longhand is also renderable
//! ([`expand_to_source`]) so the elided default is never *hidden*, only *elided* (the M-142/LSP
//! "expand ambient" projection; §5). Realization note (KC-3): the provenance is recorded at the
//! surface/resolution layer rather than as a new `mycelium_core::Provenance` variant — that would
//! change a frozen data-contract schema (`provenance.schema.json`) for metadata that is not hashed
//! (RFC-0001 §4.6) and is fully recoverable here. See the RFC-0012 changelog (append-only).

use crate::ast::{
    AmbientParams, Arm, BaseType, Ctor, Expr, FnDecl, FnSig, Item, Literal, Nodule, Paradigm,
    Param, Pattern, Scalar, Sparsity, TraitDecl, TypeDecl, TypeRef,
};

/// A never-silent refusal from the resolution pass (§4.3/§4.4) — always explicit, never a guess.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AmbientError {
    /// Two nodule-scope `default paradigm` declarations — the outer frame is ambiguous.
    MultipleDefaults {
        /// The first declared paradigm.
        first: Paradigm,
        /// The second (rejected) declaration.
        second: Paradigm,
    },
    /// A paradigm-less repr `{…}` with **no enclosing ambient** (§4.3) — there is no implicit
    /// global fallback (that would be silent).
    UnresolvedAmbient {
        /// The item/definition being resolved.
        site: String,
    },
    /// A written shape that does not fit the ambient paradigm (§4.3) — never coerced.
    ParadigmShapeMismatch {
        /// The item/definition being resolved.
        site: String,
        /// The ambient paradigm in force.
        paradigm: Paradigm,
        /// Why the written shape does not fit it.
        detail: String,
    },
    /// A bare decimal under a `Dense`/`VSA` ambient (§4.3): those paradigms have no bare-decimal
    /// encoding (the integer paradigms `Binary`/`Ternary` do).
    BareDecimalNoEncoding {
        /// The item/definition being resolved.
        site: String,
        /// The ambient paradigm in force.
        paradigm: Paradigm,
    },
}

impl core::fmt::Display for AmbientError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            AmbientError::MultipleDefaults { first, second } => write!(
                f,
                "two `default paradigm` declarations (`{first}` then `{second}`) — a nodule has one \
                 outer ambient (RFC-0012 §4.2); nest a `with paradigm` block for a local override"
            ),
            AmbientError::UnresolvedAmbient { site } => write!(
                f,
                "`{site}`: a paradigm-less repr `{{…}}` has no enclosing ambient — declare \
                 `default paradigm P` (or wrap in `with paradigm P {{ … }}`), or write the paradigm \
                 explicitly. There is no implicit global default (RFC-0012 §4.3, never-silent)"
            ),
            AmbientError::ParadigmShapeMismatch {
                site,
                paradigm,
                detail,
            } => write!(
                f,
                "`{site}`: the paradigm-less repr does not fit the `{paradigm}` ambient — {detail} \
                 (RFC-0012 §4.3; the shape is never coerced)"
            ),
            AmbientError::BareDecimalNoEncoding { site, paradigm } => write!(
                f,
                "`{site}`: a bare decimal has no `{paradigm}` encoding — only `Binary`/`Ternary` \
                 ambients give a bare decimal a meaning (RFC-0012 §4.3); write the value explicitly"
            ),
        }
    }
}

impl std::error::Error for AmbientError {}

/// A record of one ambient fill, for EXPLAIN / "where did this paradigm come from?" (§4.3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolutionNote {
    /// The item/definition the fill occurred in.
    pub site: String,
    /// The paradigm supplied by the ambient.
    pub paradigm: Paradigm,
    /// What was filled (a repr tag or a bare-decimal encoding).
    pub detail: String,
}

/// The resolved twin plus its provenance trace.
#[derive(Debug, Clone, PartialEq)]
pub struct Resolved {
    /// The longhand twin (paradigm-less forms filled; `with` blocks stripped; bare decimals tagged).
    pub nodule: Nodule,
    /// One note per ambient fill (provenance / EXPLAIN; §4.3).
    pub notes: Vec<ResolutionNote>,
}

/// Resolve a parsed [`Nodule`] to its longhand twin (RFC-0012 §4.3/§4.4). Identity on a program
/// that uses no ambient — the feature is purely additive (opt-in), so every pre-RFC-0012 program
/// passes through unchanged.
///
/// # Errors
/// Returns an [`AmbientError`] for any never-silent refusal (unresolved/unshaped ambient, a bare
/// decimal with no encoding, or a duplicate nodule default).
pub fn resolve(nodule: &Nodule) -> Result<Nodule, AmbientError> {
    resolve_report(nodule).map(|r| r.nodule)
}

/// Like [`resolve`], but also returns the provenance trace ([`ResolutionNote`]s) for EXPLAIN (§4.3).
///
/// # Errors
/// See [`resolve`].
pub fn resolve_report(nodule: &Nodule) -> Result<Resolved, AmbientError> {
    // The nodule-scope ambient: at most one `default paradigm`, the outermost frame. It governs all
    // signature types and is the base expression ambient.
    let mut default: Option<Paradigm> = None;
    for item in &nodule.items {
        if let Item::Default(p) = item {
            if let Some(first) = default {
                return Err(AmbientError::MultipleDefaults { first, second: *p });
            }
            default = Some(*p);
        }
    }

    let mut r = Resolver { notes: Vec::new() };
    let mut items = Vec::with_capacity(nodule.items.len());
    for item in &nodule.items {
        match item {
            // The default declaration is consumed (it is metadata for resolution, not a runtime
            // item); the longhand twin carries no ambient.
            Item::Default(_) => {}
            Item::Use(p) => items.push(Item::Use(p.clone())),
            Item::Type(td) => items.push(Item::Type(r.type_decl(default, td)?)),
            Item::Trait(td) => items.push(Item::Trait(r.trait_decl(default, td)?)),
            Item::Fn(fd) => items.push(Item::Fn(r.fn_decl(default, fd)?)),
        }
    }
    Ok(Resolved {
        nodule: Nodule {
            path: nodule.path.clone(),
            items,
        },
        notes: r.notes,
    })
}

/// Render a (resolved or partly-resolved) [`Nodule`] back to canonical surface text — the M-142/LSP
/// **"expand ambient"** projection (RFC-0012 §5; R12-Q3). Fed the fully-resolved twin (post-checker,
/// so bare decimals are concrete literals), it prints the exact longhand a reader would write; fed
/// the type-form-resolved nodule, it still shows every resolved *paradigm* (an `AmbientInt` renders
/// as its decimal with a paradigm note, since its width is the checker's to fill).
#[must_use]
pub fn expand_to_source(nodule: &Nodule) -> String {
    let mut out = String::new();
    out.push_str(&format!("nodule {}\n", path_str(&nodule.path)));
    for item in &nodule.items {
        out.push('\n');
        match item {
            Item::Use(p) => out.push_str(&format!("use {}\n", path_str(p))),
            Item::Default(p) => out.push_str(&format!("default paradigm {p}\n")),
            Item::Type(td) => out.push_str(&print_type_decl(td)),
            Item::Trait(td) => out.push_str(&print_trait_decl(td)),
            Item::Fn(fd) => out.push_str(&print_fn_decl(fd)),
        }
    }
    out
}

/// The resolution worker: holds the provenance trace; the ambient paradigm is threaded as an
/// argument (innermost-enclosing-wins, a binder-like stack), never as mutable state.
struct Resolver {
    notes: Vec<ResolutionNote>,
}

impl Resolver {
    fn note(&mut self, site: &str, paradigm: Paradigm, detail: impl Into<String>) {
        self.notes.push(ResolutionNote {
            site: site.to_owned(),
            paradigm,
            detail: detail.into(),
        });
    }

    fn type_decl(
        &mut self,
        amb: Option<Paradigm>,
        td: &TypeDecl,
    ) -> Result<TypeDecl, AmbientError> {
        let mut ctors = Vec::with_capacity(td.ctors.len());
        for c in &td.ctors {
            let mut fields = Vec::with_capacity(c.fields.len());
            for f in &c.fields {
                fields.push(self.type_ref(amb, &td.name, f)?);
            }
            ctors.push(Ctor {
                name: c.name.clone(),
                fields,
            });
        }
        Ok(TypeDecl {
            name: td.name.clone(),
            params: td.params.clone(),
            ctors,
        })
    }

    fn trait_decl(
        &mut self,
        amb: Option<Paradigm>,
        td: &TraitDecl,
    ) -> Result<TraitDecl, AmbientError> {
        let mut sigs = Vec::with_capacity(td.sigs.len());
        for s in &td.sigs {
            sigs.push(self.fn_sig(amb, s)?);
        }
        Ok(TraitDecl {
            name: td.name.clone(),
            params: td.params.clone(),
            sigs,
        })
    }

    fn fn_decl(&mut self, amb: Option<Paradigm>, fd: &FnDecl) -> Result<FnDecl, AmbientError> {
        let sig = self.fn_sig(amb, &fd.sig)?;
        // The function body resolves under the nodule ambient as its base frame; `with paradigm`
        // blocks nest *inside* it. Signatures (above) never see a block-scope override.
        let body = self.expr(amb, &fd.sig.name, &fd.body)?;
        Ok(FnDecl {
            thaw: fd.thaw,
            sig,
            body,
        })
    }

    fn fn_sig(&mut self, amb: Option<Paradigm>, s: &FnSig) -> Result<FnSig, AmbientError> {
        let mut value_params = Vec::with_capacity(s.value_params.len());
        for p in &s.value_params {
            value_params.push(Param {
                name: p.name.clone(),
                ty: self.type_ref(amb, &s.name, &p.ty)?,
            });
        }
        let ret = self.type_ref(amb, &s.name, &s.ret)?;
        Ok(FnSig {
            name: s.name.clone(),
            params: s.params.clone(),
            value_params,
            ret,
        })
    }

    /// Resolve a [`TypeRef`]: a paradigm-less base is filled from `amb`; everything else passes
    /// through (the guarantee index is unaffected — VR-5).
    fn type_ref(
        &mut self,
        amb: Option<Paradigm>,
        site: &str,
        t: &TypeRef,
    ) -> Result<TypeRef, AmbientError> {
        let base = match &t.base {
            BaseType::Ambient(params) => {
                let p = amb.ok_or_else(|| AmbientError::UnresolvedAmbient {
                    site: site.to_owned(),
                })?;
                let base = fill_repr(site, p, params)?;
                self.note(site, p, format!("{}", DisplayBase(&base)));
                base
            }
            // Named types may carry paradigm-less type arguments; resolve recursively.
            BaseType::Named(name, args) => {
                let mut out = Vec::with_capacity(args.len());
                for a in args {
                    out.push(self.type_ref(amb, site, a)?);
                }
                BaseType::Named(name.clone(), out)
            }
            other => other.clone(),
        };
        Ok(TypeRef {
            base,
            guarantee: t.guarantee,
        })
    }

    /// Resolve an expression under the current ambient `amb`. `with paradigm P { e }` recurses with
    /// `amb = Some(P)` and returns the resolved body (the block is stripped — I1: no node inserted).
    fn expr(&mut self, amb: Option<Paradigm>, site: &str, e: &Expr) -> Result<Expr, AmbientError> {
        Ok(match e {
            Expr::WithParadigm { paradigm, body } => self.expr(Some(*paradigm), site, body)?,
            Expr::Lit(l) => Expr::Lit(self.literal(amb, site, l)?),
            Expr::Path(p) => Expr::Path(p.clone()),
            Expr::Let {
                name,
                ty,
                bound,
                body,
            } => Expr::Let {
                name: name.clone(),
                ty: match ty {
                    Some(t) => Some(self.type_ref(amb, site, t)?),
                    None => None,
                },
                bound: Box::new(self.expr(amb, site, bound)?),
                body: Box::new(self.expr(amb, site, body)?),
            },
            Expr::If { cond, conseq, alt } => Expr::If {
                cond: Box::new(self.expr(amb, site, cond)?),
                conseq: Box::new(self.expr(amb, site, conseq)?),
                alt: Box::new(self.expr(amb, site, alt)?),
            },
            Expr::Match { scrutinee, arms } => {
                let mut out = Vec::with_capacity(arms.len());
                for arm in arms {
                    out.push(Arm {
                        pattern: self.pattern(amb, site, &arm.pattern)?,
                        body: self.expr(amb, site, &arm.body)?,
                    });
                }
                Expr::Match {
                    scrutinee: Box::new(self.expr(amb, site, scrutinee)?),
                    arms: out,
                }
            }
            Expr::For {
                x,
                xs,
                acc,
                init,
                body,
            } => Expr::For {
                x: x.clone(),
                xs: Box::new(self.expr(amb, site, xs)?),
                acc: acc.clone(),
                init: Box::new(self.expr(amb, site, init)?),
                body: Box::new(self.expr(amb, site, body)?),
            },
            Expr::Swap {
                value,
                target,
                policy,
            } => Expr::Swap {
                value: Box::new(self.expr(amb, site, value)?),
                target: self.type_ref(amb, site, target)?,
                policy: policy.clone(),
            },
            Expr::Wild(b) => Expr::Wild(Box::new(self.expr(amb, site, b)?)),
            Expr::Spore(b) => Expr::Spore(Box::new(self.expr(amb, site, b)?)),
            // A `colony`'s ambient flows transparently into each `hypha` body (no new ambient frame;
            // RFC-0008 §4.7). Resolve every hypha body under the same `amb`.
            Expr::Colony(hyphae) => {
                let mut out = Vec::with_capacity(hyphae.len());
                for h in hyphae {
                    out.push(crate::ast::Hypha {
                        body: self.expr(amb, site, &h.body)?,
                    });
                }
                Expr::Colony(out)
            }
            Expr::App { head, args } => {
                let mut out = Vec::with_capacity(args.len());
                for a in args {
                    out.push(self.expr(amb, site, a)?);
                }
                Expr::App {
                    head: Box::new(self.expr(amb, site, head)?),
                    args: out,
                }
            }
            Expr::Ascribe(inner, t) => Expr::Ascribe(
                Box::new(self.expr(amb, site, inner)?),
                self.type_ref(amb, site, t)?,
            ),
        })
    }

    /// Resolve a literal: a bare decimal under an *integer* ambient becomes [`Literal::AmbientInt`]
    /// (the checker fills its width); under `Dense`/`VSA` it is a never-silent refusal; with no
    /// ambient it stays [`Literal::Int`] (the checker's existing "no representation family" rule
    /// applies — unchanged status quo). Tagged literals are unaffected (§4.3).
    fn literal(
        &mut self,
        amb: Option<Paradigm>,
        site: &str,
        l: &Literal,
    ) -> Result<Literal, AmbientError> {
        Ok(match l {
            Literal::Int(v) => match amb {
                Some(p @ (Paradigm::Binary | Paradigm::Ternary)) => {
                    self.note(site, p, format!("bare decimal `{v}` adopts `{p}`"));
                    Literal::AmbientInt(p, *v)
                }
                Some(p) => {
                    return Err(AmbientError::BareDecimalNoEncoding {
                        site: site.to_owned(),
                        paradigm: p,
                    })
                }
                None => Literal::Int(*v),
            },
            Literal::List(elems) => {
                let mut out = Vec::with_capacity(elems.len());
                for e in elems {
                    out.push(self.expr(amb, site, e)?);
                }
                Literal::List(out)
            }
            // Tagged literals already name their paradigm; AmbientInt is only produced here.
            Literal::Bin(_) | Literal::Trit(_) | Literal::AmbientInt(_, _) => l.clone(),
        })
    }

    /// Resolve a pattern: a bare-decimal literal pattern adopts the ambient paradigm (its width is
    /// the scrutinee's, checked by `normalize_pattern`). Constructor/binder/wildcard patterns and
    /// tagged-literal patterns are unaffected.
    fn pattern(
        &mut self,
        amb: Option<Paradigm>,
        site: &str,
        p: &Pattern,
    ) -> Result<Pattern, AmbientError> {
        Ok(match p {
            Pattern::Lit(l) => Pattern::Lit(self.literal(amb, site, l)?),
            Pattern::Ctor(name, subs) => {
                let mut out = Vec::with_capacity(subs.len());
                for s in subs {
                    out.push(self.pattern(amb, site, s)?);
                }
                Pattern::Ctor(name.clone(), out)
            }
            Pattern::Wildcard | Pattern::Ident(_) => p.clone(),
        })
    }
}

/// Fill a paradigm-less `{params}` with paradigm `p` into a concrete [`BaseType`], or refuse with a
/// [`AmbientError::ParadigmShapeMismatch`] (§4.3) — the shape is never coerced.
fn fill_repr(site: &str, p: Paradigm, params: &AmbientParams) -> Result<BaseType, AmbientError> {
    let mismatch = |detail: &str| AmbientError::ParadigmShapeMismatch {
        site: site.to_owned(),
        paradigm: p,
        detail: detail.to_owned(),
    };
    Ok(match (p, params) {
        (Paradigm::Binary, AmbientParams::Size(n)) => BaseType::Binary(*n),
        (Paradigm::Ternary, AmbientParams::Size(n)) => BaseType::Ternary(*n),
        (Paradigm::Dense, AmbientParams::Dense(d, s)) => BaseType::Dense(*d, *s),
        (
            Paradigm::Vsa,
            AmbientParams::Vsa {
                model,
                dim,
                sparsity,
            },
        ) => BaseType::Vsa {
            model: model.clone(),
            dim: *dim,
            sparsity: sparsity.clone(),
        },
        (Paradigm::Binary | Paradigm::Ternary, _) => {
            return Err(mismatch("this paradigm takes a single size `{N}`"))
        }
        (Paradigm::Dense, _) => return Err(mismatch("`Dense` takes `{dim, scalar}`")),
        (Paradigm::Vsa, _) => return Err(mismatch("`VSA` takes `{model, dim, sparsity}`")),
    })
}

// --- surface pretty-printer (the "expand ambient" projection) -------------------------------------

fn path_str(p: &crate::ast::Path) -> String {
    p.0.join(".")
}

fn print_type_decl(td: &TypeDecl) -> String {
    let params = if td.params.is_empty() {
        String::new()
    } else {
        format!("<{}>", td.params.join(", "))
    };
    let ctors: Vec<String> = td
        .ctors
        .iter()
        .map(|c| {
            if c.fields.is_empty() {
                c.name.clone()
            } else {
                let fs: Vec<String> = c.fields.iter().map(print_type_ref).collect();
                format!("{}({})", c.name, fs.join(", "))
            }
        })
        .collect();
    format!("type {}{} = {}\n", td.name, params, ctors.join(" | "))
}

fn print_trait_decl(td: &TraitDecl) -> String {
    let params = if td.params.is_empty() {
        String::new()
    } else {
        format!("<{}>", td.params.join(", "))
    };
    let mut s = format!("trait {}{} {{\n", td.name, params);
    for sig in &td.sigs {
        s.push_str(&format!("  fn {}\n", print_sig_tail(sig)));
    }
    s.push_str("}\n");
    s
}

fn print_fn_decl(fd: &FnDecl) -> String {
    format!(
        "{}fn {} =\n  {}\n",
        if fd.thaw { "thaw " } else { "" },
        print_sig_tail(&fd.sig),
        print_expr(&fd.body)
    )
}

fn print_sig_tail(sig: &FnSig) -> String {
    let tp = if sig.params.is_empty() {
        String::new()
    } else {
        format!("<{}>", sig.params.join(", "))
    };
    let ps: Vec<String> = sig
        .value_params
        .iter()
        .map(|p| format!("{}: {}", p.name, print_type_ref(&p.ty)))
        .collect();
    format!(
        "{}{}({}) -> {}",
        sig.name,
        tp,
        ps.join(", "),
        print_type_ref(&sig.ret)
    )
}

fn print_type_ref(t: &TypeRef) -> String {
    let base = format!("{}", DisplayBase(&t.base));
    match t.guarantee {
        Some(g) => format!("{base} @ {g:?}"),
        None => base,
    }
}

/// A [`Display`] for [`BaseType`] in canonical surface form (shared by the printer and the
/// provenance notes).
struct DisplayBase<'a>(&'a BaseType);

impl core::fmt::Display for DisplayBase<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self.0 {
            BaseType::Binary(n) => write!(f, "Binary{{{n}}}"),
            BaseType::Ternary(m) => write!(f, "Ternary{{{m}}}"),
            BaseType::Dense(d, s) => write!(f, "Dense{{{d}, {}}}", scalar_str(*s)),
            BaseType::Vsa {
                model,
                dim,
                sparsity,
            } => write!(f, "VSA{{{model}, {dim}, {}}}", sparsity_str(sparsity)),
            BaseType::Substrate(t) => write!(f, "Substrate{{{t}}}"),
            BaseType::Named(n, args) if args.is_empty() => write!(f, "{n}"),
            BaseType::Named(n, args) => {
                let a: Vec<String> = args.iter().map(print_type_ref).collect();
                write!(f, "{n}<{}>", a.join(", "))
            }
            BaseType::Ambient(params) => write!(f, "{{{}}}", ambient_params_str(params)),
        }
    }
}

fn scalar_str(s: Scalar) -> &'static str {
    match s {
        Scalar::F16 => "F16",
        Scalar::Bf16 => "BF16",
        Scalar::F32 => "F32",
        Scalar::F64 => "F64",
    }
}

fn sparsity_str(s: &Sparsity) -> String {
    match s {
        Sparsity::Dense => "Dense".to_owned(),
        Sparsity::Sparse(k) => format!("Sparse{{{k}}}"),
    }
}

fn ambient_params_str(p: &AmbientParams) -> String {
    match p {
        AmbientParams::Size(n) => format!("{n}"),
        AmbientParams::Dense(d, s) => format!("{d}, {}", scalar_str(*s)),
        AmbientParams::Vsa {
            model,
            dim,
            sparsity,
        } => format!("{model}, {dim}, {}", sparsity_str(sparsity)),
    }
}

fn print_expr(e: &Expr) -> String {
    match e {
        Expr::Lit(l) => print_literal(l),
        Expr::Path(p) => path_str(p),
        Expr::Let {
            name,
            ty,
            bound,
            body,
        } => {
            let ann = ty
                .as_ref()
                .map(|t| format!(": {}", print_type_ref(t)))
                .unwrap_or_default();
            format!(
                "let {name}{ann} = {} in {}",
                print_expr(bound),
                print_expr(body)
            )
        }
        Expr::If { cond, conseq, alt } => format!(
            "if {} then {} else {}",
            print_expr(cond),
            print_expr(conseq),
            print_expr(alt)
        ),
        Expr::Match { scrutinee, arms } => {
            let arms: Vec<String> = arms
                .iter()
                .map(|a| format!("{} => {}", print_pattern(&a.pattern), print_expr(&a.body)))
                .collect();
            format!("match {} {{ {} }}", print_expr(scrutinee), arms.join(", "))
        }
        Expr::For {
            x,
            xs,
            acc,
            init,
            body,
        } => format!(
            "for {x} in {}, {acc} = {} => {}",
            print_expr(xs),
            print_expr(init),
            print_expr(body)
        ),
        Expr::Swap {
            value,
            target,
            policy,
        } => format!(
            "swap({}, to: {}, policy: {})",
            print_expr(value),
            print_type_ref(target),
            path_str(policy)
        ),
        Expr::WithParadigm { paradigm, body } => {
            format!("with paradigm {paradigm} {{ {} }}", print_expr(body))
        }
        Expr::Wild(b) => format!("wild {{ {} }}", print_expr(b)),
        Expr::Spore(b) => format!("spore({})", print_expr(b)),
        Expr::Colony(hyphae) => {
            let hs: Vec<String> = hyphae
                .iter()
                .map(|h| format!("hypha {}", print_expr(&h.body)))
                .collect();
            format!("colony {{ {} }}", hs.join(", "))
        }
        Expr::App { head, args } => {
            let args: Vec<String> = args.iter().map(print_expr).collect();
            format!("{}({})", print_expr(head), args.join(", "))
        }
        Expr::Ascribe(inner, t) => format!("{} : {}", print_expr(inner), print_type_ref(t)),
    }
}

fn print_pattern(p: &Pattern) -> String {
    match p {
        Pattern::Wildcard => "_".to_owned(),
        Pattern::Lit(l) => print_literal(l),
        Pattern::Ctor(n, subs) => {
            let s: Vec<String> = subs.iter().map(print_pattern).collect();
            format!("{n}({})", s.join(", "))
        }
        Pattern::Ident(n) => n.clone(),
    }
}

fn print_literal(l: &Literal) -> String {
    match l {
        Literal::Bin(s) => format!("0b{s}"),
        Literal::Trit(s) => format!("<{s}>"),
        Literal::Int(i) => format!("{i}"),
        // A still-unresolved ambient decimal: show the decimal + its resolved paradigm (the width is
        // the checker's to fill — this only appears when expanding a type-form-only nodule).
        Literal::AmbientInt(p, i) => format!("{i} /* {p} (width from context) */"),
        Literal::List(es) => {
            let s: Vec<String> = es.iter().map(print_expr).collect();
            format!("[{}]", s.join(", "))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse;

    fn nodule(src: &str) -> Nodule {
        parse(src).expect("parses")
    }

    #[test]
    fn no_ambient_is_the_identity() {
        let c = nodule("nodule d\nfn main() -> Binary{8} = not(0b1011_0010)");
        assert_eq!(resolve(&c).unwrap(), c);
    }

    #[test]
    fn a_paradigm_less_repr_is_filled_and_traced() {
        let c = nodule("nodule d\ndefault paradigm Binary\nfn main() -> {8} = 0b1011_0010");
        let r = resolve_report(&c).unwrap();
        // The `default` item is stripped; the return type is now concrete.
        assert!(!r.nodule.items.iter().any(|i| matches!(i, Item::Default(_))));
        // A provenance note records the fill (EXPLAIN: "where did this paradigm come from?").
        assert!(
            r.notes
                .iter()
                .any(|n| n.paradigm == Paradigm::Binary && n.detail.contains("Binary{8}")),
            "notes: {:?}",
            r.notes
        );
    }

    #[test]
    fn a_with_block_is_stripped_to_its_body() {
        let c = nodule(
            "nodule d\nfn main() -> Ternary{6} = with paradigm Ternary { swap(0b1011_0010, to: {6}, policy: rt) }",
        );
        let r = resolve(&c).unwrap();
        let Some(Item::Fn(fd)) = r.items.iter().find(|i| matches!(i, Item::Fn(_))) else {
            unreachable!("main is present")
        };
        // The `with paradigm` wrapper is gone; the body is the bare swap with a concrete target.
        assert!(matches!(fd.body, Expr::Swap { .. }));
    }

    #[test]
    fn multiple_defaults_are_refused() {
        let c = nodule(
            "nodule d\ndefault paradigm Binary\ndefault paradigm Ternary\nfn main() -> {8} = 0b1011_0010",
        );
        assert!(matches!(
            resolve(&c),
            Err(AmbientError::MultipleDefaults { .. })
        ));
    }

    #[test]
    fn a_shape_mismatch_is_refused() {
        let c = nodule("nodule d\ndefault paradigm Ternary\nfn main() -> {4, F32} = <0+-->");
        assert!(matches!(
            resolve(&c),
            Err(AmbientError::ParadigmShapeMismatch { .. })
        ));
    }
}
