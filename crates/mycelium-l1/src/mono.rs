//! **Monomorphization** (M-673; RFC-0007 §11.3 / §12.3, RFC-0019 §4.4) — the elaboration pre-pass
//! that turns a *checked* generic-and-trait `Env` into a **closed, monomorphic** `Env` the existing
//! [`crate::elab::elaborate`] / [`crate::elab::build_registry`] then lower **unchanged**.
//!
//! # What it does (and what it deliberately does not)
//! [`monomorphize`] re-walks the reachable graph from a nullary monomorphic `entry`, specializing
//! each generic function/data instantiation at its concrete type arguments and **statically
//! resolving** each unqualified trait-method call to the one coherent instance's method body
//! (re-emitted as a direct, mangled function). The result `Env` has **every `params` empty**, **no
//! reachable [`Ty::Var`]**, and **no trait-method calls** — so the L1-eval ≡ L0-interp ≡ AOT
//! differential (NFR-7) runs on a single closed L0 program for generics *and* traits. **No
//! `mycelium-core` change** (KC-3): this is a pure frontend rewrite over the checked `Env`; the
//! kernel/registry path is untouched.
//!
//! It is **not** a tag-changing pass (VR-5 / S1). Totality (and any grade) are **recomputed** over
//! the specialized function set, never fabricated — a specialization's verdict equals its source's
//! because the rewrite is structural. [`subst_ty`] is Swap-free; mono never inserts a `Swap`.
//!
//! # Honest identity fragmentation (NOT "one body, one hash")
//! The mangled-name scheme **is** the honest record: `first_or` specialized at `Binary{8}` and at
//! `Binary{4}` become **two distinct** functions `first_or$Binary8` and `first_or$Binary4`, each
//! with its own elaboration and content hash. This is identity *fragmentation*, recorded — not
//! hidden behind a single shared body. (Cross-instantiation sharing of structurally-identical L0
//! terms would be a separate, later content-addressing concern; mono does not claim it.)
//!
//! # Mangling: injective, surface-disjoint (`$` joints, `#` nullary-data tag)
//! Names are mangled with `$` (the joint separator) and a `#` kind-tag on a nullary data type —
//! neither is a surface-identifier character (the lexer never produces them), and the elaborator's
//! fresh variables use `%` ([`crate::elab`]). So a mangled name collides with **neither** a surface
//! name, **nor** a fresh elaboration variable, **nor across the repr/data boundary**: a data type
//! whose name happens to equal a repr mangle (e.g. a type literally named `Binary8`) tags to
//! `Binary8#`, which can never equal the repr `Binary{8}` → `Binary8`. The scheme is therefore
//! **injective** over every input it sees — distinct `(decl, type-args)` (and the repr set) map to
//! distinct names, so two instantiations never silently alias to one body (G2). A unit test pins
//! this, including the adversarial repr-named data type. **Empty type arguments ⇒ the original name,
//! byte-for-byte** (the `#` tag appears only inside a composite name; a monomorphic data type is
//! still registered and referenced under its bare name) — so monomorphic code and non-generic
//! programs pass through unchanged.
//!
//! # Still a `Residual` after M-673 (never-silent — kept explicit)
//! Mono refuses, with [`ElabError::Residual`], anything still outside the fragment: an
//! **undetermined** type parameter (a `Ty::Var` the checker would not let through, defended here too
//! — never guessed), multi-parameter traits / associated types, higher-order (`A -> B`) generics
//! (the surface is first-order — there is no function type), and `wild`/FFI, `spore`, VSA, and
//! `Substrate` (which have no v0 lowering regardless of generics). The generic/trait `Residual` sites
//! in [`crate::elab`] are **kept** as defensive internal invariants (G2): after mono they should be
//! unreachable, but they never silently disappear.

use std::collections::{BTreeMap, BTreeSet};

use crate::ast::{
    Arm, BaseType, Expr, FnDecl, FnSig, Hypha, Param, Path, Pattern, Scalar, TypeRef, WidthRef,
};
use crate::checkty::{
    has_var, infer_type, param_subst, resolve_ty, subst_ty, type_head, unify, CtorInfo, DataInfo,
    Env, TraitInfo, Ty, Width,
};
use crate::elab::ElabError;

/// A reified **instance selection** (RFC-0019 §4.4; house rule #2 — no black boxes). When mono
/// lowers a trait-method call to a direct call, it records *which* instance it picked: the trait, the
/// concrete receiver type, and the mangled name of the emitted method function. The dispatch choice
/// is thus programmatically inspectable (`EXPLAIN`-able), not hidden inside the rewrite.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstanceSelection {
    /// The trait whose method was called.
    pub trait_name: String,
    /// The concrete receiver type the instance is `for` (the full type, not the head — e.g.
    /// `Binary{8}`, never just `Binary`).
    pub for_ty: Ty,
    /// The mangled name of the monomorphic function mono emitted for this instance's method (the
    /// direct callee the trait-method call was rewritten to — e.g. `cmp$Cmp$Binary8`).
    pub impl_mangled: String,
}

/// The **EXPLAIN record** of a monomorphization (M-673): every trait-method dispatch mono resolved,
/// keyed by the mangled callee name (which itself encodes `(method, trait, receiver)`). Populated by
/// [`monomorphize_with_selections`]; queryable so the dictionary-free static resolution is a
/// reified, inspectable record rather than a black box (house rule #2).
///
/// Extended in M-687 (RFC-0024 §4) to also record **HOF defunctionalization specializations**
/// (`hof_specs`): each static HOF specialization — the source fn, its type args, its baked-in
/// function arguments, and the mangled name — is recorded for full inspectability.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MonoSelections {
    by_mangled: BTreeMap<String, InstanceSelection>,
    /// HOF defunctionalization records (RFC-0024 §4, M-687): keyed by the mangled HOF
    /// specialization name (e.g. `map$Binary8$Binary8%1:double`).
    pub(crate) hof_specs: BTreeMap<String, HofSpecialization>,
}

impl MonoSelections {
    /// The selection mono made for the mangled callee `mangled`, if any. The mangled name is what a
    /// rewritten trait-method call now refers to, so a consumer can map a direct call back to the
    /// trait/instance it came from.
    #[must_use]
    pub fn get(&self, mangled: &str) -> Option<&InstanceSelection> {
        self.by_mangled.get(mangled)
    }

    /// Every recorded selection, in deterministic (mangled-name) order. Additive read accessor.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &InstanceSelection)> {
        self.by_mangled.iter()
    }

    /// How many distinct trait-method instances were resolved (0 for a non-trait program).
    #[must_use]
    pub fn len(&self) -> usize {
        self.by_mangled.len()
    }

    /// Were no trait-method selections recorded? (A non-trait program monomorphizes with an empty
    /// record.)
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.by_mangled.is_empty()
    }

    /// The HOF defunctionalization record for the mangled specialization `mangled`, if any
    /// (RFC-0024 §4, M-687). Returns the source fn, type args, and baked-in function arguments.
    #[must_use]
    pub fn hof(&self, mangled: &str) -> Option<&HofSpecialization> {
        self.hof_specs.get(mangled)
    }

    /// Every recorded HOF specialization, in deterministic (mangled-name) order.
    pub fn hof_iter(&self) -> impl Iterator<Item = (&String, &HofSpecialization)> {
        self.hof_specs.iter()
    }
}

/// Monomorphize a checked `Env` from nullary monomorphic `entry`, returning a closed monomorphic
/// `Env` the existing [`crate::elab::elaborate`] runs unchanged.
///
/// On a program with **no** generics/traits this is a fast **pass-through** (a clone): monomorphic
/// code is mono's identity, so the pre-M-673 differential corpus is observably unchanged (NFR-7).
///
/// # Errors
/// [`ElabError::Residual`] for anything outside the monomorphizable fragment (an undetermined type
/// parameter, a multi-parameter trait, a higher-order generic, …) — never silent, never a guess
/// (G2/VR-5). [`ElabError::UnknownFn`] if `entry` is absent.
pub fn monomorphize(env: &Env, entry: &str) -> Result<Env, ElabError> {
    monomorphize_with_selections(env, entry).map(|(env, _)| env)
}

/// Like [`monomorphize`] but also returns the [`MonoSelections`] EXPLAIN record of every trait-method
/// dispatch resolved (house rule #2 — the static resolution is inspectable, not a black box).
///
/// # Errors
/// See [`monomorphize`].
pub fn monomorphize_with_selections(
    env: &Env,
    entry: &str,
) -> Result<(Env, MonoSelections), ElabError> {
    // Fast pass-through: a fully-monomorphic, non-trait program is mono's identity. Returning a clone
    // keeps the existing monomorphic differential corpus byte-identical (NFR-7) and avoids re-walking
    // a graph that has nothing to specialize.
    if is_already_monomorphic(env) {
        return Ok((env.clone(), MonoSelections::default()));
    }
    let mut m = Mono::new(env);
    m.run(entry)?;
    Ok(m.finish())
}

/// Is `env` already fully monomorphic, trait-free, **and** HOF-free? Then mono is the identity
/// (the fast pass-through). True iff **no** function is generic, **no** function has a fn-typed
/// value parameter (which needs defunctionalization — RFC-0024 §4, M-687), **no** data type is
/// generic, and there are **no** traits / instances / retained impls.
fn is_already_monomorphic(env: &Env) -> bool {
    env.fns.values().all(|fd| {
        fd.sig.params.is_empty()
            && fd
                .sig
                .value_params
                .iter()
                .all(|p| !param_has_fn_type(&env.types, &fd.sig.param_names(), &p.ty))
    }) && env.types.values().all(|d| d.params.is_empty())
        && env.traits.is_empty()
        && env.instances.is_empty()
        && env.impls.is_empty()
}

/// True iff the parameter type `t` resolves to (or contains) a `Ty::Fn` — meaning this parameter
/// is a HOF that needs defunctionalization (RFC-0024 §4, M-687). Best-effort: a resolution failure
/// is treated as "not fn-typed" (the full mono pass will catch it with an explicit Residual).
fn param_has_fn_type(
    types: &BTreeMap<String, crate::checkty::DataInfo>,
    tyvars: &[String],
    t: &TypeRef,
) -> bool {
    use crate::ast::BaseType;
    match &t.base {
        BaseType::Fn(_, _) => true,
        BaseType::Named(n, args) => {
            // A type variable or a data type with fn-typed arguments — check args recursively.
            // A data type itself (not a type var) is not fn-typed; a type variable is also not
            // fn-typed at the surface level (it resolves to a concrete type at specialization time,
            // which may or may not be `Ty::Fn`; we conservatively say false here and let the full
            // mono pass handle it with an explicit Residual if it turns out fn-typed).
            if tyvars.contains(n) || types.contains_key(n.as_str()) {
                return false; // bare type var or concrete data type — not a fn type itself
            }
            // Otherwise check args (e.g. `F<A->B>` would have a fn-typed arg — exotic but safe).
            args.iter().any(|a| param_has_fn_type(types, tyvars, a))
        }
        _ => false, // Binary/Ternary/Dense/Substrate — never fn-typed
    }
}

/// A monomorphization work item — the unit of the dedup worklist. Deduplication is by the item's
/// canonical [`item_key`] (a discriminant-tagged mangled string), so a `BTreeSet<String>` of seen
/// keys guarantees each specialization is emitted **once** (dedup ⟹ the recursive walk terminates).
#[derive(Debug, Clone, PartialEq, Eq)]
enum Item {
    /// A function instance: the source fn `name` at concrete type arguments `targs` (empty for a
    /// monomorphic fn — which mangles to `name` unchanged), optionally specialised by resolved
    /// **function-argument** identities (RFC-0024 §4, M-687). `fn_args` carries `(param_index,
    /// callee_mangled_name)` for each value-parameter whose type is `Ty::Fn`; empty means no
    /// higher-order specialization. An `Item::Fn` with non-empty `fn_args` is a defunctionalized
    /// HOF specialization — distinct from the un-specialized (or differently-specialized) version
    /// of the same fn.
    Fn {
        name: String,
        targs: Vec<Ty>,
        /// Resolved width arguments in declaration order (DN-42 / M-753 step-c): one `Width::Lit`
        /// per width parameter of the callee. Baked into the item key so two calls at different
        /// widths produce distinct specializations (never a silent alias — G2/VR-5).
        wargs: Vec<Width>,
        /// `(param_index, callee_mangled_name)` for each fn-typed value parameter, sorted by
        /// param index (deterministic). Baked into the item key so two different function
        /// arguments produce two distinct specializations (never a silent alias — G2).
        fn_args: Vec<(usize, String)>,
    },
    /// A data-type instance: the source type `name` at concrete `targs`.
    Data { name: String, targs: Vec<Ty> },
    /// A trait-method instance: the unqualified method `method` of trait `trait_name`, resolved at the
    /// concrete receiver `for_ty` (the coherent instance's method, emitted as a direct fn).
    Method {
        trait_name: String,
        method: String,
        for_ty: Ty,
    },
}

/// The **EXPLAIN record** of a single HOF defunctionalization (RFC-0024 §4, M-687): which
/// higher-order function was specialized, at which type arguments, with which function arguments
/// baked in. Recorded in [`MonoSelections`] so the static dispatch is inspectable (house rule #2
/// — no black boxes).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HofSpecialization {
    /// The source (polymorphic / HOF) function name.
    pub source_fn: String,
    /// The concrete type arguments (empty if the HOF was monomorphic).
    pub targs: Vec<Ty>,
    /// The resolved function argument(s): `(param_index, callee_mangled_name)`, parallel to the
    /// `fn_args` of the [`Item::Fn`] that triggered this specialization.
    pub fn_args: Vec<(usize, String)>,
    /// The mangled name of the emitted closed-first-order specialization.
    pub mangled: String,
}

/// The monomorphization driver: the source (checked, generic) env, the dedup worklist, and the
/// accumulating monomorphic output (`fns`/`types`) plus the EXPLAIN selection record.
struct Mono<'e> {
    src: &'e Env,
    /// Canonical keys of items already enqueued (dedup) — guarantees one emission per specialization
    /// (termination). Keyed by [`item_key`] so `Ty` needs no `Ord` (it is `Eq` only).
    seen: BTreeSet<String>,
    /// The pending worklist (LIFO; order does not affect the result — emission is keyed by mangled
    /// name into `BTreeMap`s).
    work: Vec<Item>,
    /// Emitted monomorphic functions (mangled name → closed `FnDecl`).
    out_fns: BTreeMap<String, FnDecl>,
    /// Emitted monomorphic data types (mangled name → `DataInfo` with empty `params`).
    out_types: BTreeMap<String, DataInfo>,
    /// The reified trait-method dispatch record (house rule #2).
    selections: BTreeMap<String, InstanceSelection>,
    /// HOF defunctionalization specialization records (RFC-0024 §4, M-687; house rule #2).
    hof_specs: BTreeMap<String, HofSpecialization>,
    /// Active fn-parameter substitution during HOF specialization emission (M-687): maps a
    /// value-parameter name whose type is `Ty::Fn` to the mangled name of its resolved callee.
    /// Populated by [`emit_fn`] when `fn_args` is non-empty; cleared after each emission. Only
    /// consulted in [`rewrite_hof_app`].
    fn_param_subst: BTreeMap<String, String>,
}

impl<'e> Mono<'e> {
    fn new(src: &'e Env) -> Self {
        Mono {
            src,
            seen: BTreeSet::new(),
            work: Vec::new(),
            out_fns: BTreeMap::new(),
            out_types: BTreeMap::new(),
            selections: BTreeMap::new(),
            hof_specs: BTreeMap::new(),
            fn_param_subst: BTreeMap::new(),
        }
    }

    /// Enqueue an item if it has not been seen (dedup ⟹ termination).
    fn enqueue(&mut self, item: Item) {
        if self.seen.insert(item_key(&item)) {
            self.work.push(item);
        }
    }

    /// Seed from the nullary monomorphic `entry` and drain the worklist, specializing each item.
    fn run(&mut self, entry: &str) -> Result<(), ElabError> {
        let Some(fd) = self.src.fns.get(entry) else {
            return Err(ElabError::UnknownFn(entry.to_owned()));
        };
        if !fd.sig.params.is_empty() {
            return residual(
                entry,
                "monomorphization entry is generic — elaborate a concrete (nullary, monomorphic) \
                 entry (RFC-0007 §11.3)",
            );
        }
        self.enqueue(Item::Fn {
            name: entry.to_owned(),
            targs: vec![],
            wargs: vec![], // entry is monomorphic (nullary, no width params)
            fn_args: vec![],
        });
        while let Some(item) = self.work.pop() {
            match item {
                Item::Fn {
                    name,
                    targs,
                    wargs,
                    fn_args,
                } => self.emit_fn(&name, &targs, &wargs, &fn_args)?,
                Item::Data { name, targs } => self.emit_data(&name, &targs)?,
                Item::Method {
                    trait_name,
                    method,
                    for_ty,
                } => self.emit_method(&trait_name, &method, &for_ty)?,
            }
        }
        Ok(())
    }

    /// Consume the driver into the closed monomorphic [`Env`] plus its [`MonoSelections`] EXPLAIN
    /// record: the emitted fns/types, recomputed totality, and **empty** trait/instance/impl
    /// registries (no generics/traits remain).
    fn finish(self) -> (Env, MonoSelections) {
        // Recompute totality over the specialized fn set (a specialization's verdict equals its
        // source's; the SCC/descent machinery is structural — totality.rs). The matured gate and the
        // elaborator's SCC pass then read verdicts by the *mangled* names. Never fabricated (VR-5).
        let totality = crate::totality::classify_all(&self.out_fns);
        let env = Env {
            types: self.out_types,
            fns: self.out_fns,
            totality,
            traits: BTreeMap::new(),
            instances: BTreeMap::new(),
            impls: BTreeMap::new(),
            // DN-54 / M-812: monomorphized specializations do not carry lower rules (the rule
            // registry is a pre-mono artefact — rules are expanded before/at elaborate time).
            lower_rules: BTreeMap::new(),
        };
        (
            env,
            MonoSelections {
                by_mangled: self.selections,
                hof_specs: self.hof_specs,
            },
        )
    }

    /// Specialize source function `name` at concrete `targs` (and optionally with baked-in
    /// function arguments `fn_args` for HOF defunctionalization — RFC-0024 §4, M-687) and emit
    /// the monomorphic `FnDecl` under its mangled name. Discovers transitive instances by walking
    /// (and rewriting) the body.
    ///
    /// When `fn_args` is non-empty: each `(param_index, callee_mangled)` pair names a
    /// value-parameter whose declared type is `Ty::Fn` and the statically-resolved callee that was
    /// passed at the call site. The specialized body replaces every application of that fn-param
    /// with a direct call to the callee, and the fn-param is **dropped** from the emitted
    /// value-parameter list — producing a closed first-order signature (no `Ty::Fn` in params).
    fn emit_fn(
        &mut self,
        name: &str,
        targs: &[Ty],
        wargs: &[Width],
        fn_args: &[(usize, String)],
    ) -> Result<(), ElabError> {
        let mangled = mangle_hof_decl(name, targs, wargs, fn_args);
        // Already emitted? (the worklist dedups, but a defensive check keeps emission idempotent.)
        if self.out_fns.contains_key(&mangled) {
            return Ok(());
        }
        let fd = self
            .src
            .fns
            .get(name)
            .ok_or_else(|| ElabError::UnknownFn(name.to_owned()))?
            .clone();
        let tyvars = fd.sig.param_names();
        if tyvars.len() != targs.len() {
            return residual(
                name,
                format!(
                    "internal: `{name}` has {} type parameter(s) but was queued with {} argument(s)",
                    tyvars.len(),
                    targs.len()
                ),
            );
        }
        let mut subst: BTreeMap<String, Ty> = param_subst(&tyvars, targs);

        // DN-42 / M-753 step-c: inject width-arg carriers into the shared subst map so that
        // `subst_ty` resolves `Width::Var(v)` in parameter/return types to the concrete literal.
        // Carrier convention: `var_name → Ty::Binary(Width::Lit(n))` regardless of paradigm —
        // `subst_ty` extracts the right paradigm (Binary or Ternary) from the carrier. An
        // undetermined width var at emit time is an internal invariant violation (VR-5).
        let wvars = fd.sig.width_param_names();
        if wvars.len() != wargs.len() {
            return residual(
                name,
                format!(
                    "internal: `{name}` has {} width parameter(s) but was queued with {} \
                     width argument(s) — an invariant violation (DN-42 / M-753 step-c)",
                    wvars.len(),
                    wargs.len()
                ),
            );
        }
        for (v, w) in wvars.iter().zip(wargs.iter()) {
            match w {
                Width::Lit(n) => {
                    subst.insert(v.clone(), Ty::Binary(Width::Lit(*n)));
                }
                Width::Var(wv) => {
                    return residual(
                        name,
                        format!(
                            "width param `{v}` of `{name}` is still a variable `{wv}` at emit \
                             — undetermined width is never guessed (DN-42 §4 / VR-5)"
                        ),
                    );
                }
            }
        }

        // Build the fn-parameter substitution map for HOF defunctionalization:
        //   fn_param_name → callee_mangled_name
        // and validate that each fn-arg index names an actual fn-typed param.
        let fn_arg_map: BTreeMap<String, String> = fn_args
            .iter()
            .map(|(idx, callee)| {
                let pname = fd
                    .sig
                    .value_params
                    .get(*idx)
                    .map(|p| p.name.clone())
                    .ok_or_else(|| ElabError::Residual {
                        site: name.to_owned(),
                        what: format!(
                            "HOF fn_arg index {idx} out of bounds for `{name}` (internal)"
                        ),
                    })?;
                Ok((pname, callee.clone()))
            })
            .collect::<Result<_, ElabError>>()?;

        // Set of param indices that are fn-typed and will be dropped from the emitted signature.
        let dropped_indices: BTreeSet<usize> = fn_args.iter().map(|(i, _)| *i).collect();

        // The concrete value-parameter scope (param name → substituted concrete type), for
        // re-inferring sub-expression types while walking the body. Fn-typed params that are being
        // defunctionalized are added to scope at their `Ty::Fn` type (so re-inference still works),
        // but are **not** emitted in `new_params` (they are dropped from the closed-first-order sig).
        let mut scope: Vec<(String, Ty)> = Vec::with_capacity(fd.sig.value_params.len());
        let mut new_params: Vec<Param> = Vec::with_capacity(fd.sig.value_params.len());
        for (idx, p) in fd.sig.value_params.iter().enumerate() {
            let cty = self.concrete_ty(name, &tyvars, &subst, &p.ty)?;
            // Enqueue any generic data instance the parameter type names, so a type that appears
            // only as a parameter (never destructured in this body) is still emitted (insurance;
            // dedup makes it idempotent). Skip for Ty::Fn — no data enqueuing needed.
            if !matches!(cty, Ty::Fn(_, _)) {
                self.enqueue_tys_in(&cty);
            }
            scope.push((p.name.clone(), cty.clone()));
            // Drop fn-typed params that are being defunctionalized from the emitted signature.
            if !dropped_indices.contains(&idx) {
                new_params.push(Param {
                    name: p.name.clone(),
                    ty: ty_to_ref(&cty),
                });
            }
        }
        let ret_cty = self.concrete_ty(name, &tyvars, &subst, &fd.sig.ret)?;
        self.enqueue_tys_in(&ret_cty);

        // Install the HOF fn-param substitution map for the duration of this body rewrite.
        debug_assert!(
            self.fn_param_subst.is_empty(),
            "fn_param_subst must be empty before entering emit_fn (invariant)"
        );
        self.fn_param_subst = fn_arg_map;

        // The declared return type drives return-position inference (e.g. a bare nullary generic
        // ctor, or a return-driven trait-method receiver), mirroring the checker's `expected`.
        let body_result = self.rewrite(name, &mut scope, &fd.body, Some(&ret_cty));

        // Always clear the substitution map — even on error.
        self.fn_param_subst.clear();

        let new_body = body_result?;
        let new_sig = FnSig {
            name: mangled.clone(),
            params: vec![], // monomorphic: no type parameters remain
            value_params: new_params,
            ret: ty_to_ref(&ret_cty),
            effects: fd.sig.effects.clone(),
        };
        self.out_fns.insert(
            mangled.clone(),
            FnDecl {
                vis: fd.vis,
                thaw: fd.thaw,
                tier: fd.tier,
                sig: new_sig,
                body: new_body,
            },
        );
        // EXPLAIN: if this was a HOF specialization, record it (house rule #2 — no black boxes).
        if !fn_args.is_empty() {
            self.hof_specs.insert(
                mangled.clone(),
                HofSpecialization {
                    source_fn: name.to_owned(),
                    targs: targs.to_vec(),
                    fn_args: fn_args.to_vec(),
                    mangled,
                },
            );
        }
        Ok(())
    }

    /// Specialize source data type `name` at concrete `targs` and emit the monomorphic [`DataInfo`]
    /// (empty `params`; fields rewritten to mangled-nullary `Ty::Data`). Constructor names are mangled
    /// so distinct instantiations never collide on a ctor name (the registry/`Env::ctor` key).
    fn emit_data(&mut self, name: &str, targs: &[Ty]) -> Result<(), ElabError> {
        let mangled = mangle_decl(name, targs);
        if self.out_types.contains_key(&mangled) {
            return Ok(());
        }
        let d = self
            .src
            .types
            .get(name)
            .ok_or_else(|| ElabError::Residual {
                site: name.to_owned(),
                what: format!("unknown data type `{name}` during monomorphization"),
            })?
            .clone();
        if d.params.len() != targs.len() {
            return residual(
                name,
                format!(
                    "internal: data `{name}` has {} type parameter(s) but was queued with {}",
                    d.params.len(),
                    targs.len()
                ),
            );
        }
        let subst = param_subst(&d.params, targs);
        let mut ctors: Vec<CtorInfo> = Vec::with_capacity(d.ctors.len());
        for c in &d.ctors {
            let mut fields: Vec<Ty> = Vec::with_capacity(c.fields.len());
            for f in &c.fields {
                let cf = subst_ty(f, &subst);
                if has_var(&cf) {
                    return residual(
                        name,
                        format!(
                            "data `{name}` field stays abstract ({cf}) after substitution — an \
                             undetermined type parameter is never guessed (RFC-0007 §11.3)"
                        ),
                    );
                }
                // Enqueue any data instance the field references, and rewrite it to its mangled-nullary
                // form so the registry/`field_spec` consumes the already-working `Ty::Data(n, [])` arm.
                self.enqueue_tys_in(&cf);
                fields.push(mangle_ty_in_ty(&cf));
            }
            ctors.push(CtorInfo {
                name: mangle_ctor(&c.name, targs),
                fields,
            });
        }
        self.out_types.insert(
            mangled.clone(),
            DataInfo {
                name: mangled,
                params: vec![],
                ctors,
            },
        );
        Ok(())
    }

    /// Statically resolve trait `trait_name`'s method `method` at concrete receiver `for_ty` and emit
    /// the instance's resolved method body as a direct monomorphic fn under the mangled method name.
    /// Records the [`InstanceSelection`] (EXPLAIN). The instance was confirmed during checking
    /// (`require_instance`), so resolution here is deterministic — never a guess (G2).
    fn emit_method(
        &mut self,
        trait_name: &str,
        method: &str,
        for_ty: &Ty,
    ) -> Result<(), ElabError> {
        let mangled = mangle_method(method, trait_name, for_ty);
        if self.out_fns.contains_key(&mangled) {
            return Ok(());
        }
        let Some(head) = type_head(for_ty) else {
            return residual(
                method,
                format!(
                    "trait-method receiver `{for_ty}` has no concrete instance head — a blanket / \
                     abstract receiver is not a stage-1 instance (RFC-0019 §4.5)"
                ),
            );
        };
        let key = (trait_name.to_owned(), head);
        let methods = self
            .src
            .impls
            .get(&key)
            .ok_or_else(|| ElabError::Residual {
                site: method.to_owned(),
                what: format!(
                "no retained impl methods for `({trait_name}, {for_ty})` — the instance was not \
                 found during monomorphization (RFC-0019 §4.5 / M-673)"
            ),
            })?;
        // Resolution must match the FULL receiver (head-erasure is the coherence key, not the
        // resolution key — a `Binary{8}` instance must not serve a `Binary{4}` call; G2). The retained
        // instance's concrete `for_ty` is on record in `src.instances`.
        if let Some(info) = self.src.instance(trait_name, &key.1) {
            if info.for_ty != *for_ty {
                return residual(
                    method,
                    format!(
                        "the `{trait_name}` instance on this head is for `{}`, not `{for_ty}` — \
                         never a silently reused mismatched instance (RFC-0019 §4.5)",
                        info.for_ty
                    ),
                );
            }
        }
        let md = methods
            .iter()
            .find(|m| m.sig.name == method)
            .ok_or_else(|| ElabError::Residual {
                site: method.to_owned(),
                what: format!("instance `({trait_name}, {for_ty})` has no method `{method}`"),
            })?
            .clone();
        // An impl method over a concrete `for_ty` carries no abstract type-variables (the checker
        // resolved its param/return types concretely), so the empty substitution is correct; we still
        // route through `concrete_ty` to defend the no-`Ty::Var` invariant.
        let empty: BTreeMap<String, Ty> = BTreeMap::new();
        let mut scope: Vec<(String, Ty)> = Vec::with_capacity(md.sig.value_params.len());
        let mut new_params: Vec<Param> = Vec::with_capacity(md.sig.value_params.len());
        for p in &md.sig.value_params {
            let cty = self.concrete_ty(method, &[], &empty, &p.ty)?;
            self.enqueue_tys_in(&cty);
            scope.push((p.name.clone(), cty.clone()));
            new_params.push(Param {
                name: p.name.clone(),
                ty: ty_to_ref(&cty),
            });
        }
        let ret_cty = self.concrete_ty(method, &[], &empty, &md.sig.ret)?;
        self.enqueue_tys_in(&ret_cty);
        let new_body = self.rewrite(method, &mut scope, &md.body, Some(&ret_cty))?;
        self.out_fns.insert(
            mangled.clone(),
            FnDecl {
                vis: md.vis,
                thaw: md.thaw,
                tier: md.tier,
                sig: FnSig {
                    name: mangled.clone(),
                    params: vec![],
                    value_params: new_params,
                    ret: ty_to_ref(&ret_cty),
                    effects: md.sig.effects.clone(),
                },
                body: new_body,
            },
        );
        // EXPLAIN: record the resolved selection, keyed by the mangled callee (which encodes
        // method+trait+receiver). Inspectable, not a black box (house rule #2).
        self.selections.insert(
            mangled.clone(),
            InstanceSelection {
                trait_name: trait_name.to_owned(),
                for_ty: for_ty.clone(),
                impl_mangled: mangled,
            },
        );
        Ok(())
    }

    /// Resolve a declared [`TypeRef`] (with the decl's type-params as vars) to its **concrete** [`Ty`]
    /// under `subst`, refusing if a `Ty::Var` survives (an undetermined parameter — never guessed).
    fn concrete_ty(
        &self,
        site: &str,
        tyvars: &[String],
        subst: &BTreeMap<String, Ty>,
        t: &TypeRef,
    ) -> Result<Ty, ElabError> {
        let abstract_ty =
            resolve_ty(site, &self.src.types, tyvars, t).map_err(|e| ElabError::Residual {
                site: site.to_owned(),
                what: format!("could not resolve a type during monomorphization: {e}"),
            })?;
        let c = subst_ty(&abstract_ty.0, subst);
        if has_var(&c) {
            return residual(
                site,
                format!(
                    "type `{c}` stays abstract after substitution — an undetermined type parameter \
                     is never guessed (RFC-0007 §11.3 / S1)"
                ),
            );
        }
        // The concrete type may itself name a generic data instance to enqueue (e.g. `List<Binary{8}>`
        // as a parameter type).
        Ok(c)
    }

    /// Enqueue every generic **data** instance mentioned in a concrete `Ty` (recursing into
    /// arguments), so a type used only inside another type/field is still emitted.
    fn enqueue_tys_in(&mut self, ty: &Ty) {
        if let Ty::Data(n, args) = ty {
            for a in args {
                self.enqueue_tys_in(a);
            }
            // A monomorphic (nullary) data type still needs registering if it is reachable; enqueue it
            // either way (empty targs mangle to the original name, so it is byte-identical).
            if self.src.types.contains_key(n) {
                self.enqueue(Item::Data {
                    name: n.clone(),
                    targs: args.clone(),
                });
            }
        }
    }

    // ----- the body rewriter -------------------------------------------------------------------

    /// Rewrite (and walk) an expression under a **concrete** value scope, threading the bidirectional
    /// `expected` type. Mirrors every [`Expr`] arm: rewrites `App`/`Path`/`Pattern` names to their
    /// mangled monomorphic forms, discovers transitive instances, and refuses anything outside the
    /// monomorphizable fragment with an explicit [`ElabError::Residual`] (never silent — G2).
    ///
    /// `expected` matters where the checker's bidirectional pass used it: a bare nullary generic ctor
    /// (`Nil`) and a return-driven trait-method receiver both take their type from context.
    fn rewrite(
        &mut self,
        site: &str,
        scope: &mut Vec<(String, Ty)>,
        e: &Expr,
        expected: Option<&Ty>,
    ) -> Result<Expr, ElabError> {
        match e {
            Expr::Lit(l) => Ok(Expr::Lit(l.clone())),
            Expr::Path(p) => self.rewrite_path(site, scope, p, expected),
            Expr::App { head, args } => self.rewrite_app(site, scope, head, args, expected),
            Expr::Let {
                name,
                ty,
                bound,
                body,
            } => {
                // The bound's expected is its ascription (if any), resolved concretely; the body's is
                // the enclosing `expected`. The binder's concrete type comes from re-inference.
                let want = match ty {
                    Some(t) => Some(self.concrete_ty(site, &[], &BTreeMap::new(), t)?),
                    None => None,
                };
                let bound2 = self.rewrite(site, scope, bound, want.as_ref())?;
                let bty = self.infer(site, scope, bound)?;
                scope.push((name.clone(), bty));
                let body2 = self.rewrite(site, scope, body, expected);
                scope.pop();
                let body2 = body2?;
                Ok(Expr::Let {
                    name: name.clone(),
                    // The ascription, if present, is now concrete (mono erases type params); keep it
                    // for fidelity (the elaborator ignores the type part — it re-infers).
                    ty: want.as_ref().map(ty_to_ref),
                    bound: Box::new(bound2),
                    body: Box::new(body2),
                })
            }
            Expr::If { cond, conseq, alt } => {
                let bool_ty = Ty::Data("Bool".to_owned(), vec![]);
                let cond2 = self.rewrite(site, scope, cond, Some(&bool_ty))?;
                let conseq2 = self.rewrite(site, scope, conseq, expected)?;
                // The else-branch may borrow the then-branch's type as its expected (bare-decimal
                // width sharing), mirroring `check_if`.
                let then_ty = self.infer(site, scope, conseq)?;
                let alt2 = self.rewrite(site, scope, alt, expected.or(Some(&then_ty)))?;
                Ok(Expr::If {
                    cond: Box::new(cond2),
                    conseq: Box::new(conseq2),
                    alt: Box::new(alt2),
                })
            }
            Expr::Match { scrutinee, arms } => {
                self.rewrite_match(site, scope, scrutinee, arms, expected)
            }
            Expr::For {
                x,
                xs,
                acc,
                init,
                body,
            } => self.rewrite_for(site, scope, x, xs, acc, init, body),
            Expr::Swap {
                value,
                target,
                policy,
            } => {
                // `swap` is never silent; mono does not touch its certificate. The target is a concrete
                // repr (no type params), kept verbatim; only the value is rewritten.
                let value2 = self.rewrite(site, scope, value, None)?;
                Ok(Expr::Swap {
                    value: Box::new(value2),
                    target: target.clone(),
                    policy: policy.clone(),
                })
            }
            Expr::Ascribe(inner, t) => {
                let want = self.concrete_ty(site, &[], &BTreeMap::new(), t)?;
                let inner2 = self.rewrite(site, scope, inner, Some(&want))?;
                Ok(Expr::Ascribe(Box::new(inner2), ty_to_ref(&want)))
            }
            Expr::Colony(hyphae) => {
                let mut out = Vec::with_capacity(hyphae.len());
                for h in hyphae {
                    out.push(Hypha {
                        body: self.rewrite(site, scope, &h.body, None)?,
                    });
                }
                Ok(Expr::Colony(out))
            }
            // DN-58 §A/§B (M-667): `fuse(a, b)` and `reclaim(policy) { body }` — rewrite both
            // operands/policy/body through monomorphization. These constructs are type-concrete
            // (the checker verified homogeneity); any lingering type-variable inside an operand
            // is a monomorphization concern handled transparently here.
            Expr::Fuse { left, right } => {
                // DN-58 §A.5 (M-817): a **Data**-type `fuse(a, b)` desugars to the resolved
                // `Fuse::join` trait-method call — exactly the form the L1 evaluator dispatches
                // (`eval.rs` builds `join(left, right)`), and the form that makes the user merge
                // **run** three-way (the coherent instance method is emitted as a direct fn and
                // inlined by `elab`). A **repr**-type `fuse` has no user `join`; its meet is a
                // built-in (the `Binary` meet is `fuse_join:binary`), so it stays an `Expr::Fuse`
                // for `elab` to lower to the meet prim. The checker (`check_fuse`) has already
                // verified a coherent `Fuse` instance exists for the Data case, so the resolution
                // below cannot be a guess (G2/VR-5).
                let lty = self.infer(site, scope, left)?;
                let is_repr = matches!(
                    &lty,
                    Ty::Binary(_) | Ty::Ternary(_) | Ty::Dense(_, _) | Ty::Bytes | Ty::Seq(_, _)
                );
                if is_repr {
                    let left2 = self.rewrite(site, scope, left, None)?;
                    let right2 = self.rewrite(site, scope, right, None)?;
                    Ok(Expr::Fuse {
                        left: Box::new(left2),
                        right: Box::new(right2),
                    })
                } else {
                    // `fuse(a, b) ≡ join(a, b)` (left ↦ `self`, right ↦ `other` — DN-58 §A.2
                    // canonical `Fuse::join`). Route through the trait-method resolver so the
                    // coherent instance is *selected and recorded* (EXPLAIN — house rule #2), never
                    // guessed. The expected type seeds return-driven receiver inference; the operand
                    // types pin it regardless.
                    let join_args = [left.as_ref().clone(), right.as_ref().clone()];
                    self.rewrite_trait_method_call(site, scope, "join", &join_args, expected)
                }
            }
            Expr::Reclaim { policy, body } => {
                let policy2 = self.rewrite(site, scope, policy, None)?;
                let body2 = self.rewrite(site, scope, body, expected)?;
                Ok(Expr::Reclaim {
                    policy: Box::new(policy2),
                    body: Box::new(body2),
                })
            }
            // Constructs with no v0 lowering regardless of generics — kept as explicit residuals so the
            // elaborator's own refusal still fires (defense in depth; never a fabricated artifact).
            Expr::Wild(_) => residual(
                site,
                "wild/FFI has no L0 form in v0 — monomorphization does not change that (M-661)",
            ),
            Expr::Spore(_) => residual(site, "`spore` is deferred (E2-5/M-260)"),
            // M-664: `consume` of a `Substrate` has no L0 form in v0 (LR-8) — monomorphization does
            // not change that; an explicit residual (defense in depth) mirrors the elaborator's
            // refusal, never a fabricated artifact (G2).
            Expr::Consume(_) => residual(
                site,
                "`consume` of an affine `Substrate` has no L0 form in v0 (LR-8; DN-03 §1; M-664)",
            ),
            Expr::Lambda { .. } => residual(
                site,
                "`lambda` (closures) is deferred to M-704 / RFC-0024 §5 (RFC-0037 D5 reserves the surface)",
            ),
            Expr::WithParadigm { .. } => residual(
                site,
                "internal: a `with paradigm` block reached monomorphization — the ambient \
                 resolution pass strips it before checking (RFC-0012 §4.4)",
            ),
        }
    }

    /// Rewrite a path/variable. A local binder passes through; a recursive-fn reference or a nullary
    /// constructor is rewritten to its mangled monomorphic name (and its instance enqueued).
    fn rewrite_path(
        &mut self,
        site: &str,
        scope: &[(String, Ty)],
        p: &Path,
        expected: Option<&Ty>,
    ) -> Result<Expr, ElabError> {
        if p.0.len() != 1 {
            return residual(site, format!("dotted path `{}`", p.0.join(".")));
        }
        let name = &p.0[0];
        // A value binder in scope is left as-is.
        if scope.iter().any(|(n, _)| n == name) {
            return Ok(Expr::Path(p.clone()));
        }
        // A nullary data constructor (Nil, Z, True, …). Its type — hence its data instance — comes from
        // `expected` for a generic type (mirroring `check_path`); a monomorphic one needs no context.
        if let Some((d, i)) = self.src.ctor(name) {
            if d.ctors[i].fields.is_empty() {
                let (dname, targs) = self.ctor_data_instance(site, &d.name, expected)?;
                self.enqueue(Item::Data {
                    name: dname.clone(),
                    targs: targs.clone(),
                });
                return Ok(Expr::Path(Path(vec![mangle_ctor(name, &targs)])));
            }
            // A non-nullary ctor referenced bare is unsaturated — the checker already refused it; keep
            // an explicit residual as defense in depth.
            return residual(
                site,
                format!("constructor `{name}` referenced without saturation (W6)"),
            );
        }
        // A bare reference to a (recursive) function. Monomorphic fns mangle to themselves; a generic
        // fn cannot be referenced as a bare value in the first-order surface (it would need arguments).
        if let Some(fd) = self.src.fns.get(name) {
            if !fd.sig.params.is_empty() {
                return residual(
                    site,
                    format!(
                        "generic function `{name}` referenced as a bare value — the surface is \
                         first-order (no function values); apply it (RFC-0007 §11.3)"
                    ),
                );
            }
            self.enqueue(Item::Fn {
                name: name.clone(),
                targs: vec![],
                wargs: vec![], // monomorphic path reference: no type or width params
                fn_args: vec![],
            });
            return Ok(Expr::Path(Path(vec![mangle_decl(name, &[])])));
        }
        // Unresolved here means a free name; the checker would have refused it. Keep it verbatim so the
        // elaborator's own "unresolved name" residual fires (never silently dropped).
        Ok(Expr::Path(p.clone()))
    }

    /// Rewrite an application head + arguments. Dispatches exactly as the checker's `check_app`:
    /// user fn (monomorphic or generic), constructor (monomorphic or generic), unqualified
    /// trait-method, or prim — rewriting names to mangled forms and enqueuing instances.
    fn rewrite_app(
        &mut self,
        site: &str,
        scope: &mut Vec<(String, Ty)>,
        head: &Expr,
        args: &[Expr],
        expected: Option<&Ty>,
    ) -> Result<Expr, ElabError> {
        let Expr::Path(p) = head else {
            return residual(site, "v0 application head must be a name (first-order)");
        };
        if p.0.len() != 1 {
            return residual(site, format!("dotted call `{}`", p.0.join(".")));
        }
        let name = &p.0[0];

        // (0) HOF parameter application: `f(x)` where `f` is a fn-typed value parameter being
        // defunctionalized. The fn-param substitution map maps `f` to its resolved callee's mangled
        // name — rewrite to a direct call (RFC-0024 §4, M-687). The callee was already enqueued
        // when the HOF specialization was enqueued at the outer call site.
        if let Some(callee_mangled) = self.fn_param_subst.get(name).cloned() {
            // The HOF parameter `f: A -> B` is single-argument (RFC-0024 §3/§5 — multi-arg is a
            // staged Residual). Validate the argument count to stay never-silent (G2).
            if args.len() != 1 {
                return residual(
                    site,
                    format!(
                        "HOF parameter `{name}` applied to {} argument(s); only 1 is supported in \
                         stage-1 (RFC-0024 §5 — partial application / multi-arg HOF is deferred)",
                        args.len()
                    ),
                );
            }
            // Re-infer the arg type from scope to thread the right `expected` (mirrors the
            // checker's `Ty::Fn` arm in `check_app`). The callee must already be in `out_fns`
            // (it was enqueued by `rewrite_app` at the outer HOF call site and emitted before the
            // HOF body is walked — if not, an `emit_fn` is triggered now via the worklist; since
            // the worklist drains recursively the callee is present). For re-inference we can use
            // `None` as `expected` (the arg type is concrete from scope).
            let arg2 = self.rewrite(site, scope, &args[0], None)?;
            return Ok(Expr::App {
                head: Box::new(Expr::Path(Path(vec![callee_mangled]))),
                args: vec![arg2],
            });
        }

        // (1) User function call (the head name is in scope as a fn). Clone the decl so the immutable
        // borrow of `self.src` does not outlive the `&mut self` calls below.
        if let Some(fd) = self.src.fns.get(name).cloned() {
            // DN-42 / M-753 step-c: call infer_fn_targs if the function has either
            // type params OR width params. Both return types are bundled together.
            let (targs, wargs) =
                if fd.sig.params.is_empty() && fd.sig.width_param_names().is_empty() {
                    (vec![], vec![])
                } else {
                    self.infer_fn_targs(site, scope, name, &fd, args)?
                };
            // Detect fn-typed value parameters and resolve their actual arguments
            // (RFC-0024 §4, M-687 static defunctionalization).
            let fn_args = self.resolve_fn_args(site, scope, name, &fd, &targs, args)?;
            let want_tys = self.fn_value_param_tys(site, &fd, &targs)?;
            // Rewrite only the non-fn-typed arguments (fn-typed args are baked into the
            // specialization key and dropped from the call; they are not emitted in args2).
            let fn_arg_indices: BTreeSet<usize> = fn_args.iter().map(|(i, _)| *i).collect();
            let mut args2 = Vec::with_capacity(args.len());
            for (idx, (a, exp)) in args.iter().zip(want_tys.iter()).enumerate() {
                if fn_arg_indices.contains(&idx) {
                    // This argument is a function value — it is baked into the specialization key
                    // and not passed at the call site (defunctionalized away). Skip it.
                    // But still enqueue the callee fn so it is emitted (it may not be reachable
                    // from any other path).
                    continue;
                }
                args2.push(self.rewrite(site, scope, a, Some(exp))?);
            }
            let mangled = mangle_hof_decl(name, &targs, &wargs, &fn_args);
            self.enqueue(Item::Fn {
                name: name.clone(),
                targs: targs.clone(),
                wargs: wargs.clone(),
                fn_args,
            });
            return Ok(Expr::App {
                head: Box::new(Expr::Path(Path(vec![mangled]))),
                args: args2,
            });
        }

        // (2) Saturated constructor application.
        if let Some((d, _)) = self.src.ctor(name) {
            let dname = d.name.clone();
            // The concrete data instance of this constructor application — `infer_type` types the whole
            // app to `Ty::Data(dname, targs)` (it solves the data targs from the field args + expected).
            // `app_ctor_data_instance` resolves only via the `n == dname` arm, so its data name is
            // always `dname`; keep just the solved type args (the owner name is already known).
            let (_di, targs) =
                self.app_ctor_data_instance(site, scope, head, args, &dname, expected)?;
            // Rewrite each field argument under its concrete field-type expected.
            let field_tys = self.ctor_field_tys(site, &dname, name, &targs)?;
            let args2 = self.rewrite_call_args(site, scope, field_tys, args)?;
            self.enqueue(Item::Data {
                name: dname,
                targs: targs.clone(),
            });
            return Ok(Expr::App {
                head: Box::new(Expr::Path(Path(vec![mangle_ctor(name, &targs)]))),
                args: args2,
            });
        }

        // (3) Unqualified trait-method call (resolved to a direct call to the instance method).
        if self.is_trait_method(name) {
            return self.rewrite_trait_method_call(site, scope, name, args, expected);
        }

        // (4) A prim (or an unknown name the elaborator will refuse). Rewrite arguments and keep the
        //     head verbatim — prims have no type parameters. A bare-decimal arg is already resolved by
        //     the checker, so each arg infers concretely.
        let mut args2 = Vec::with_capacity(args.len());
        for a in args {
            args2.push(self.rewrite(site, scope, a, None)?);
        }
        Ok(Expr::App {
            head: Box::new(head.clone()),
            args: args2,
        })
    }

    /// Solve a generic **function** call's type arguments by unifying the callee's declared parameter
    /// types (abstract over its type-params) against the actual argument types (re-inferred concretely
    /// in the current scope) — exactly the checker's `check_app_generic_fn` inference. An undetermined
    /// parameter is an explicit residual (never guessed — G2/VR-5).
    fn infer_fn_targs(
        &self,
        site: &str,
        scope: &mut Vec<(String, Ty)>,
        name: &str,
        fd: &FnDecl,
        args: &[Expr],
    ) -> Result<(Vec<Ty>, Vec<Width>), ElabError> {
        if fd.sig.value_params.len() != args.len() {
            return residual(
                site,
                format!(
                    "`{name}` takes {} argument(s), got {}",
                    fd.sig.value_params.len(),
                    args.len()
                ),
            );
        }
        let callee_vars = fd.sig.param_names();
        let mut subst: BTreeMap<String, Ty> = BTreeMap::new();
        for (pm, a) in fd.sig.value_params.iter().zip(args) {
            let want = resolve_ty(site, &self.src.types, &callee_vars, &pm.ty)
                .map_err(|e| res_err(site, e))?
                .0;
            let want_now = subst_ty(&want, &subst);
            let got = self.infer(site, scope, a)?;
            unify_into(site, &want_now, &got, &mut subst)?;
        }
        let mut targs = Vec::with_capacity(callee_vars.len());
        for v in &callee_vars {
            match subst.get(v) {
                Some(t) if !has_var(t) => targs.push(t.clone()),
                _ => {
                    return residual(
                        site,
                        format!(
                            "`{name}` is generic over `{v}`, but this call does not determine it — \
                             never a guessed default (RFC-0007 §11.3 / VR-5)"
                        ),
                    )
                }
            }
        }
        // DN-42 / M-753 step-c: also collect resolved width arguments (carrier convention —
        // width var `N` was bound as `Ty::Binary(Width::Lit(n))` by unify). An unresolved width
        // parameter is an explicit residual — never a guessed default (VR-5/G2).
        let callee_wvars = fd.sig.width_param_names();
        let mut wargs = Vec::with_capacity(callee_wvars.len());
        for v in &callee_wvars {
            match subst.get(v) {
                Some(Ty::Binary(Width::Lit(n))) => wargs.push(Width::Lit(*n)),
                _ => {
                    return residual(
                        site,
                        format!(
                            "`{name}` is width-generic over `{v}`, but this call does not \
                             determine the width — undetermined width is never guessed (DN-42 §4 / VR-5)"
                        ),
                    )
                }
            }
        }
        Ok((targs, wargs))
    }

    /// The concrete data instance `(dname, targs)` of a **nullary** constructor used as a value — from
    /// `expected` for a generic type (mirroring `check_path`). A monomorphic type needs no context.
    fn ctor_data_instance(
        &self,
        site: &str,
        dname: &str,
        expected: Option<&Ty>,
    ) -> Result<(String, Vec<Ty>), ElabError> {
        let d = self
            .src
            .types
            .get(dname)
            .ok_or_else(|| ElabError::Residual {
                site: site.to_owned(),
                what: format!("unknown data type `{dname}`"),
            })?;
        if d.params.is_empty() {
            return Ok((dname.to_owned(), vec![]));
        }
        match expected {
            Some(Ty::Data(en, eargs)) if en == dname && eargs.len() == d.params.len() => {
                for a in eargs {
                    if has_var(a) {
                        return residual(
                            site,
                            format!("nullary constructor of `{dname}<…>` resolved to abstract {a}"),
                        );
                    }
                }
                Ok((dname.to_owned(), eargs.clone()))
            }
            _ => residual(
                site,
                format!(
                    "constructor of generic `{dname}<…>` needs its type argument(s) from context — \
                     never a guess (RFC-0007 §11.3)"
                ),
            ),
        }
    }

    /// The concrete data instance of a **saturated** constructor application — `infer_type` types the
    /// whole application to `Ty::Data(dname, targs)`, solving the data type arguments from the field
    /// arguments (and `expected`). The returned name is the source data name; `targs` are concrete.
    fn app_ctor_data_instance(
        &self,
        site: &str,
        scope: &mut Vec<(String, Ty)>,
        head: &Expr,
        args: &[Expr],
        dname: &str,
        expected: Option<&Ty>,
    ) -> Result<(String, Vec<Ty>), ElabError> {
        let app = Expr::App {
            head: Box::new(head.clone()),
            args: args.to_vec(),
        };
        // Re-infer against `expected` so a bare nullary generic sub-ctor (`Nil`) in a field is pinned.
        let ty = self.infer_against(site, scope, &app, expected)?;
        match ty {
            Ty::Data(n, targs) if n == dname => {
                for a in &targs {
                    if has_var(a) {
                        return residual(
                            site,
                            format!("constructor `{dname}` left type argument {a} undetermined"),
                        );
                    }
                }
                Ok((n, targs))
            }
            other => residual(
                site,
                format!("constructor application did not type to `{dname}<…>` (got {other})"),
            ),
        }
    }

    /// The (substituted, concrete) value-parameter types of fn `fd` at `targs` — the per-argument
    /// `expected` types for rewriting a generic/monomorphic function call's arguments.
    fn fn_value_param_tys(
        &self,
        site: &str,
        fd: &FnDecl,
        targs: &[Ty],
    ) -> Result<Vec<Ty>, ElabError> {
        let tyvars = fd.sig.param_names();
        let subst = param_subst(&tyvars, targs);
        let mut out = Vec::with_capacity(fd.sig.value_params.len());
        for p in &fd.sig.value_params {
            let (abstract_ty, _) =
                resolve_ty(site, &self.src.types, &tyvars, &p.ty).map_err(|e| res_err(site, e))?;
            out.push(subst_ty(&abstract_ty, &subst));
        }
        Ok(out)
    }

    /// The (substituted, concrete) field types of constructor `cname` of data `dname` at `targs` —
    /// the per-argument `expected` types for rewriting the field arguments.
    fn ctor_field_tys(
        &self,
        site: &str,
        dname: &str,
        cname: &str,
        targs: &[Ty],
    ) -> Result<Vec<Ty>, ElabError> {
        let d = self
            .src
            .types
            .get(dname)
            .ok_or_else(|| ElabError::Residual {
                site: site.to_owned(),
                what: format!("unknown data type `{dname}`"),
            })?;
        let c = d
            .ctors
            .iter()
            .find(|c| c.name == cname)
            .ok_or_else(|| ElabError::Residual {
                site: site.to_owned(),
                what: format!("`{dname}` has no constructor `{cname}`"),
            })?;
        let subst = param_subst(&d.params, targs);
        Ok(c.fields.iter().map(|f| subst_ty(f, &subst)).collect())
    }

    /// Rewrite each call argument under its concrete `expected` field/parameter type (so a bare
    /// nullary generic ctor argument is pinned). `want_tys` is parallel to `args`.
    fn rewrite_call_args(
        &mut self,
        site: &str,
        scope: &mut Vec<(String, Ty)>,
        want_tys: Vec<Ty>,
        args: &[Expr],
    ) -> Result<Vec<Expr>, ElabError> {
        let mut out = Vec::with_capacity(args.len());
        for (i, a) in args.iter().enumerate() {
            let exp = want_tys.get(i);
            out.push(self.rewrite(site, scope, a, exp)?);
        }
        Ok(out)
    }

    /// Resolve and rewrite an **unqualified trait-method call** to a direct call to the coherent
    /// instance's (mangled) method (RFC-0019 §4.4). Mirrors `check_trait_method_call`: find the single
    /// owning trait, solve its parameter by unifying the method signature against the arguments
    /// (seeded from `expected`), look up the instance, enqueue + emit the method, and record the
    /// EXPLAIN selection. Refuses (never guesses) on ambiguity, a multi-parameter trait, an
    /// undetermined receiver, or a missing instance.
    fn rewrite_trait_method_call(
        &mut self,
        site: &str,
        scope: &mut Vec<(String, Ty)>,
        name: &str,
        args: &[Expr],
        expected: Option<&Ty>,
    ) -> Result<Expr, ElabError> {
        let owners: Vec<&TraitInfo> = self
            .src
            .traits
            .values()
            .filter(|tr| tr.sigs.iter().any(|s| s.name == name))
            .collect();
        let tr = match owners.as_slice() {
            [one] => *one,
            [] => {
                return residual(site, format!("`{name}` is not a trait method (internal)"));
            }
            many => {
                let names: Vec<&str> = many.iter().map(|t| t.name.as_str()).collect();
                return residual(
                    site,
                    format!(
                        "ambiguous trait-method call `{name}` — declared by multiple traits ({}) — \
                         an explicit refusal, never a guess (RFC-0019 §4.4)",
                        names.join(", ")
                    ),
                );
            }
        };
        if tr.params.len() != 1 {
            return residual(
                site,
                format!(
                    "trait-method resolution for `{name}` needs a single-parameter trait \
                     (multi-parameter traits are v2 — RFC-0019 §10)"
                ),
            );
        }
        let sig = tr
            .sigs
            .iter()
            .find(|s| s.name == name)
            .expect("owner has the method");
        if sig.value_params.len() != args.len() {
            return residual(
                site,
                format!(
                    "trait method `{}::{name}` takes {} argument(s), got {}",
                    tr.name,
                    sig.value_params.len(),
                    args.len()
                ),
            );
        }
        let tparam = &tr.params[0];
        let trait_vars = std::slice::from_ref(tparam);
        let mut subst: BTreeMap<String, Ty> = BTreeMap::new();
        // Seed from `expected` against the (abstract) return type — return-driven receiver inference
        // (mirrors `check_trait_method_call`).
        if let Some(exp) = expected {
            if let Ok((ret_abs, _)) = resolve_ty(site, &self.src.types, trait_vars, &sig.ret) {
                let _ = unify_into(site, &ret_abs, exp, &mut subst);
            }
        }
        for (pm, a) in sig.value_params.iter().zip(args) {
            let want = resolve_ty(site, &self.src.types, trait_vars, &pm.ty)
                .map_err(|e| res_err(site, e))?
                .0;
            let want_now = subst_ty(&want, &subst);
            let got = self.infer(site, scope, a)?;
            unify_into(site, &want_now, &got, &mut subst)?;
        }
        let Some(receiver) = subst.get(tparam).cloned() else {
            return residual(
                site,
                format!(
                    "trait-method call `{name}` does not determine trait `{}`'s parameter `{tparam}` \
                     — never a guess (RFC-0019 §4.4)",
                    tr.name
                ),
            );
        };
        if has_var(&receiver) {
            return residual(
                site,
                format!(
                    "trait-method call `{name}` left receiver `{receiver}` abstract — an \
                     undetermined trait parameter is never guessed (RFC-0019 §4.4 / VR-5)"
                ),
            );
        }
        // Rewrite the arguments under the instance method's concrete parameter types.
        let want_tys: Vec<Ty> = sig
            .value_params
            .iter()
            .map(|pm| {
                resolve_ty(site, &self.src.types, trait_vars, &pm.ty)
                    .map(|(t, _)| subst_ty(&t, &subst))
                    .map_err(|e| res_err(site, e))
            })
            .collect::<Result<_, _>>()?;
        let args2 = self.rewrite_call_args(site, scope, want_tys, args)?;
        let mangled = mangle_method(name, &tr.name, &receiver);
        self.enqueue(Item::Method {
            trait_name: tr.name.clone(),
            method: name.to_owned(),
            for_ty: receiver.clone(),
        });
        Ok(Expr::App {
            head: Box::new(Expr::Path(Path(vec![mangled]))),
            args: args2,
        })
    }

    /// Resolve the **function-argument identities** for a call to `name` at `targs`, for any
    /// value-parameter whose (substituted) type is `Ty::Fn` (RFC-0024 §4, M-687). For each such
    /// parameter, the corresponding actual argument must be an `Expr::Path` naming a statically-
    /// known top-level function — anything else is a never-silent `Residual` (G2):
    /// - a closure / lambda: out of scope (RFC-0024 §5)
    /// - a fn value from a `match` / data field / fn return: dynamic — not statically resolvable
    /// - a generic function referenced bare (still-generic callee): deferred (FLAG)
    ///
    /// Returns `(param_index, callee_mangled_name)` pairs, sorted by index (deterministic). Also
    /// enqueues each resolved callee as an `Item::Fn` so it is emitted even if unreachable from
    /// other paths.
    fn resolve_fn_args(
        &mut self,
        site: &str,
        _scope: &[(String, Ty)], // available for diagnostics
        callee_name: &str,
        fd: &FnDecl,
        targs: &[Ty],
        args: &[Expr],
    ) -> Result<Vec<(usize, String)>, ElabError> {
        let tyvars = fd.sig.param_names();
        let subst = param_subst(&tyvars, targs);
        let mut fn_args: Vec<(usize, String)> = Vec::new();
        for (idx, (pm, actual)) in fd.sig.value_params.iter().zip(args).enumerate() {
            let (abstract_ty, _) =
                resolve_ty(site, &self.src.types, &tyvars, &pm.ty).map_err(|e| res_err(site, e))?;
            let cty = subst_ty(&abstract_ty, &subst);
            if !matches!(cty, Ty::Fn(_, _)) {
                continue; // not a fn-typed parameter — nothing to defunctionalize
            }
            // This parameter has type `Ty::Fn(a, b)` — the actual argument must be a statically-
            // known top-level function name (RFC-0024 §4/§5).
            let Expr::Path(p) = actual else {
                return residual(
                    site,
                    format!(
                        "call `{callee_name}` argument #{idx} (parameter `{}`) has function type \
                         `{cty}` — the actual argument must be a top-level function name (a path), \
                         not a complex expression; closures / dynamic fn values are deferred \
                         (RFC-0024 §5 — never a silent coercion)",
                        pm.name
                    ),
                );
            };
            if p.0.len() != 1 {
                return residual(
                    site,
                    format!(
                        "function-valued argument `{}` is a dotted path — only top-level \
                         (single-segment) names are first-class function values in stage-1 \
                         (RFC-0024 §3, never a silent coercion)",
                        p.0.join(".")
                    ),
                );
            }
            let fn_name = &p.0[0];
            // M-715 (recursive-HOF re-pass): `fn_name` may be a HOF VALUE PARAMETER already bound to a
            // static specialization in the current emit scope (`fn_param_subst`) — e.g. the recursive
            // call `map(rest, f)` inside `map`'s defunctionalized body re-passes the fn-param `f`, and
            // `foldl(rest, f, f(h))` re-passes `f` alongside applying it. Thread it through as the SAME
            // specialization the outer call pinned (RFC-0024 §4; that callee fn was already enqueued at
            // the outer site, so no re-enqueue is needed). Checked BEFORE the top-level lookup so a
            // parameter correctly shadows any same-named top-level fn (lexical scope). Never a silent
            // guess (G2): an unbound fn-valued name still falls through to the explicit residual below.
            if let Some(mangled) = self.fn_param_subst.get(fn_name) {
                fn_args.push((idx, mangled.clone()));
                continue;
            }
            let callee_fd = self
                .src
                .fns
                .get(fn_name)
                .ok_or_else(|| ElabError::Residual {
                    site: site.to_owned(),
                    what: format!(
                    "function-valued argument `{fn_name}` for parameter `{}` of `{callee_name}` \
                     is not a top-level function in scope — only named top-level functions are \
                     first-class values in stage-1 (RFC-0024 §3)",
                    pm.name
                ),
                })?;
            // A still-generic function as a value argument is deferred (FLAG — RFC-0024 §5).
            if !callee_fd.sig.params.is_empty() {
                return residual(
                    site,
                    format!(
                        "function-valued argument `{fn_name}` for parameter `{}` of `{callee_name}` \
                         is still generic (has type parameters) — a generic fn as a value requires \
                         type-argument context to defunctionalize; this case is deferred \
                         (RFC-0024 §5, FLAG: generic-fn-as-arg — never a silent guess)",
                        pm.name
                    ),
                );
            }
            // Monomorphic callee — enqueue it and record the resolved identity.
            let callee_mangled = mangle_decl(fn_name, &[]);
            self.enqueue(Item::Fn {
                name: fn_name.clone(),
                targs: vec![],
                wargs: vec![], // callee is monomorphic here (no width params)
                fn_args: vec![],
            });
            fn_args.push((idx, callee_mangled));
        }
        // Sort by index for determinism (already in index order since we iterate params in order,
        // but make it explicit).
        fn_args.sort_by_key(|(i, _)| *i);
        Ok(fn_args)
    }

    /// Rewrite a `match` — re-infer the (concrete) scrutinee type, rewrite the scrutinee, then each
    /// arm with its pattern's constructor names mangled and its binders bound at their concrete types.
    fn rewrite_match(
        &mut self,
        site: &str,
        scope: &mut Vec<(String, Ty)>,
        scrutinee: &Expr,
        arms: &[Arm],
        expected: Option<&Ty>,
    ) -> Result<Expr, ElabError> {
        let sty = self.infer(site, scope, scrutinee)?;
        let scrut2 = self.rewrite(site, scope, scrutinee, None)?;
        let mut out_arms = Vec::with_capacity(arms.len());
        for arm in arms {
            // Bind the pattern's variables at their concrete types (from the scrutinee type), rewrite
            // ctor names, then rewrite the arm body under the extended scope.
            let mut arm_scope = scope.clone();
            let pat2 = self.rewrite_pattern(site, &arm.pattern, &sty, &mut arm_scope)?;
            let body2 = self.rewrite(site, &mut arm_scope, &arm.body, expected)?;
            out_arms.push(Arm {
                pattern: pat2,
                body: body2,
            });
        }
        Ok(Expr::Match {
            scrutinee: Box::new(scrut2),
            arms: out_arms,
        })
    }

    /// Rewrite a pattern against the (concrete) scrutinee type `sty`: mangle each constructor name to
    /// its monomorphic form, recurse into sub-patterns at the constructor's substituted field types,
    /// and push every binder onto `scope` at its concrete type. Enqueues the data instance the pattern
    /// matches (so a pattern-only-used type is still emitted).
    fn rewrite_pattern(
        &mut self,
        site: &str,
        pat: &Pattern,
        sty: &Ty,
        scope: &mut Vec<(String, Ty)>,
    ) -> Result<Pattern, ElabError> {
        match pat {
            Pattern::Wildcard => Ok(Pattern::Wildcard),
            Pattern::Lit(l) => Ok(Pattern::Lit(l.clone())),
            Pattern::Ident(b) => {
                // A bare identifier is a binder (a nullary ctor would have been normalized to
                // `Ctor(b, [])` by the checker's `normalize_pattern` before elaboration; but `match`
                // bodies in the *source* `Env` are the resolved bodies, so a nullary ctor may appear as
                // `Ctor` already — here treat a bare ident as a binder of the scrutinee type).
                scope.push((b.clone(), sty.clone()));
                Ok(Pattern::Ident(b.clone()))
            }
            Pattern::Ctor(cname, subs) => {
                let (dname, targs) = match sty {
                    Ty::Data(n, a) => (n.clone(), a.clone()),
                    other => {
                        return residual(
                            site,
                            format!(
                                "a constructor pattern `{cname}` against non-data type {other}"
                            ),
                        )
                    }
                };
                self.enqueue(Item::Data {
                    name: dname.clone(),
                    targs: targs.clone(),
                });
                let field_tys = self.ctor_field_tys(site, &dname, cname, &targs)?;
                if field_tys.len() != subs.len() {
                    return residual(
                        site,
                        format!(
                            "constructor pattern `{cname}` binds {} field(s), the type has {}",
                            subs.len(),
                            field_tys.len()
                        ),
                    );
                }
                let mut subs2 = Vec::with_capacity(subs.len());
                for (sub, fty) in subs.iter().zip(&field_tys) {
                    subs2.push(self.rewrite_pattern(site, sub, fty, scope)?);
                }
                Ok(Pattern::Ctor(mangle_ctor(cname, &targs), subs2))
            }
        }
    }

    /// Rewrite a `for x in xs, acc = init => body` — re-infer the (concrete) spine + accumulator
    /// types, bind `x`/`acc`, and rewrite each part. The element type is the spine's element type.
    #[allow(clippy::too_many_arguments)]
    fn rewrite_for(
        &mut self,
        site: &str,
        scope: &mut Vec<(String, Ty)>,
        x: &str,
        xs: &Expr,
        acc: &str,
        init: &Expr,
        body: &Expr,
    ) -> Result<Expr, ElabError> {
        let sty = self.infer(site, scope, xs)?;
        let Ty::Data(tname, targs) = &sty else {
            return residual(site, format!("`for` spine is not a data type: {sty}"));
        };
        self.enqueue(Item::Data {
            name: tname.clone(),
            targs: targs.clone(),
        });
        let elem_ty = self.for_elem_ty(site, tname, targs)?;
        let aty = self.infer(site, scope, init)?;
        let xs2 = self.rewrite(site, scope, xs, None)?;
        let init2 = self.rewrite(site, scope, init, None)?;
        let mut body_scope = scope.clone();
        body_scope.push((x.to_owned(), elem_ty));
        body_scope.push((acc.to_owned(), aty));
        let body2 = self.rewrite(site, &mut body_scope, body, None)?;
        Ok(Expr::For {
            x: x.to_owned(),
            xs: Box::new(xs2),
            acc: acc.to_owned(),
            init: Box::new(init2),
            body: Box::new(body2),
        })
    }

    /// The element type of a linear-recursive spine type `tname` at `targs` — the single non-spine
    /// field of its cons constructor, with the type arguments substituted in.
    fn for_elem_ty(&self, site: &str, tname: &str, targs: &[Ty]) -> Result<Ty, ElabError> {
        let d = self
            .src
            .types
            .get(tname)
            .ok_or_else(|| ElabError::Residual {
                site: site.to_owned(),
                what: format!("unknown type `{tname}`"),
            })?;
        let subst = param_subst(&d.params, targs);
        for c in &d.ctors {
            if c.fields.is_empty() {
                continue;
            }
            let elem = c
                .fields
                .iter()
                .find(|f| !matches!(f, Ty::Data(n, _) if n == tname));
            if let Some(e) = elem {
                return Ok(subst_ty(e, &subst));
            }
        }
        residual(site, format!("`for` type `{tname}` has no element field"))
    }

    // ----- re-inference helpers ----------------------------------------------------------------

    /// Is `name` a method of some registered trait (the trait-method dispatch gate)?
    fn is_trait_method(&self, name: &str) -> bool {
        self.src
            .traits
            .values()
            .any(|tr| tr.sigs.iter().any(|s| s.name == name))
    }

    /// Re-infer the concrete type of `e` under the concrete `scope`, using the checker's re-inference
    /// (`infer_type`) over the *source* env. A failure is an explicit residual (never silent).
    fn infer(&self, site: &str, scope: &mut Vec<(String, Ty)>, e: &Expr) -> Result<Ty, ElabError> {
        infer_type(self.src, scope, e).map_err(|err| ElabError::Residual {
            site: site.to_owned(),
            what: format!("could not re-infer a type during monomorphization: {err}"),
        })
    }

    /// Re-infer `e` against an `expected` type (bidirectional) — needed where a bare nullary generic
    /// ctor or a return-driven receiver takes its type from context. Falls back to `infer` when there
    /// is no expected type. Uses the public bidirectional check via a temporary ascription so the
    /// `expected` is threaded without exposing the checker's private `Cx`.
    fn infer_against(
        &self,
        site: &str,
        scope: &mut Vec<(String, Ty)>,
        e: &Expr,
        expected: Option<&Ty>,
    ) -> Result<Ty, ElabError> {
        match expected {
            None => self.infer(site, scope, e),
            Some(exp) => {
                // Thread `expected` by ascribing `e : exp` and inferring that — `check_ascribe` runs the
                // bidirectional check against `exp` (so a bare `Nil` field is pinned), then returns the
                // ascribed type. `exp` is the **source-named** concrete type (re-inference resolves names
                // against the source env), and it is concrete, so the ascription is exact (never a
                // coercion — S1).
                let ascribed = Expr::Ascribe(Box::new(e.clone()), ty_to_source_ref(exp));
                self.infer(site, scope, &ascribed)
            }
        }
    }
}

// ----- free helpers ----------------------------------------------------------------------------

/// The canonical dedup key of a work item — a kind-tagged string so a function and a data type that
/// happen to mangle to the same name never alias, and `Ty` needs no `Ord` (just its `Display`).
fn item_key(item: &Item) -> String {
    match item {
        Item::Fn {
            name,
            targs,
            wargs,
            fn_args,
        } => format!("fn:{}", mangle_hof_decl(name, targs, wargs, fn_args)),
        Item::Data { name, targs } => format!("data:{}", mangle_decl(name, targs)),
        Item::Method {
            trait_name,
            method,
            for_ty,
        } => format!("method:{}", mangle_method(method, trait_name, for_ty)),
    }
}

/// Mangle a HOF-specialization declaration name at concrete type arguments **and** fn arguments
/// (RFC-0024 §4, M-687). Extends [`mangle_decl`]: after the type-argument segments (`$`-joined),
/// appends fn-argument segments as `%{param_index}:{callee_mangled}` per baked-in fn parameter.
///
/// The `%` separator is the elaborator's fresh-variable character (never a surface-identifier
/// character), so a HOF-specialization mangled name is **disjoint** from:
/// - surface names (no `$`/`#`/`%` in the Mycelium lexer)
/// - trait-method mangled names (`method$Trait$ForTy` — no `%`)
/// - type-only specializations (`name$TyArg…` — no `%`)
/// - data-repr names (no `%`)
///
/// This preserves the overall injective, surface-disjoint property of the mangling scheme (G2).
///
/// **Empty `fn_args` delegates to [`mangle_decl`]** — so a fn with no HOF params produces the
/// exact same mangled name as before M-687 (backward-compatible with the existing corpus).
pub(crate) fn mangle_hof_decl(
    name: &str,
    targs: &[Ty],
    wargs: &[Width],
    fn_args: &[(usize, String)],
) -> String {
    // DN-42 / M-753 step-c: include width arguments in the mangled name so two calls at different
    // widths produce distinct specializations (identity fragmentation; G2 / never-silent).
    // Width args are appended after type args using the same `$` joint; Width::Lit(n) becomes
    // `Binary{n}` via mangle_ty (consistent with type-arg mangling). Width::Var should never
    // reach here (mono refuses undetermined params first).
    let base = mangle_decl_with_wargs(name, targs, wargs);
    if fn_args.is_empty() {
        return base;
    }
    let mut s = base;
    for (idx, callee) in fn_args {
        s.push('%');
        s.push_str(&idx.to_string());
        s.push(':');
        s.push_str(callee);
    }
    s
}

/// Mangle a declaration name at concrete type arguments **and** width arguments (DN-42 / M-753
/// step-c). Width args are appended after type args using `$` joints:
/// `add<N>` at N=8 → `add$Binary8`. Width::Var should never reach here.
fn mangle_decl_with_wargs(name: &str, targs: &[Ty], wargs: &[Width]) -> String {
    let mut s = mangle_decl(name, targs);
    for w in wargs {
        s.push('$');
        match w {
            Width::Lit(n) => s.push_str(&format!("Binary{n}")),
            Width::Var(v) => s.push_str(&format!("WVAR_{v}")), // should not reach here (VR-5)
        }
    }
    s
}

fn residual<T>(site: &str, what: impl Into<String>) -> Result<T, ElabError> {
    Err(ElabError::Residual {
        site: site.to_owned(),
        what: what.into(),
    })
}

/// Wrap a checker [`crate::checkty::CheckError`] as an elaboration [`ElabError::Residual`] (the
/// re-inference primitives return `CheckError`; mono surfaces them as residuals — never silent).
fn res_err(site: &str, e: crate::checkty::CheckError) -> ElabError {
    ElabError::Residual {
        site: site.to_owned(),
        what: format!("monomorphization re-inference: {e}"),
    }
}

/// One-sided unification (the checker's [`crate::checkty::unify`]) surfacing its failure as a
/// residual. Binds the abstract `decl`'s type-vars from the concrete `actual`.
fn unify_into(
    site: &str,
    decl: &Ty,
    actual: &Ty,
    s: &mut BTreeMap<String, Ty>,
) -> Result<(), ElabError> {
    unify(site, decl, actual, s).map_err(|e| res_err(site, e))
}

/// Mangle a type to a flat identifier-suffix fragment (injective; `$`-free for primitives, `$`-joined
/// for applied data). `Binary{8}`→`Binary8`, `Ternary{6}`→`Ternary6`, `Dense{16,F32}`→`Dense16F32`,
/// `Data("List",[Binary8])`→`List$Binary8`, nullary `Data("Bool",[])`→`Bool`.
pub(crate) fn mangle_ty(t: &Ty) -> String {
    match t {
        Ty::Binary(Width::Lit(n)) => format!("Binary{n}"),
        Ty::Binary(Width::Var(v)) => format!("BinaryVAR_{v}"),
        Ty::Ternary(Width::Lit(m)) => format!("Ternary{m}"),
        Ty::Ternary(Width::Var(v)) => format!("TernaryVAR_{v}"),
        Ty::Dense(d, s) => format!("Dense{d}{}", scalar_tag(*s)),
        Ty::Substrate(tag) => format!("Substrate{tag}"),
        // RFC-0032 D3/D4: `Seq{T, N}` mangles to `SeqN$<elem>` (injective — the `$` separates the
        // length from the recursively-mangled element); `Bytes` is nullary.
        Ty::Seq(elem, n) => format!("Seq{n}${}", mangle_ty(elem)),
        Ty::Bytes => "Bytes".to_owned(),
        // A nullary data type tags its name with `#` (not a surface-identifier char — the lexer
        // never produces it), so a data type whose name happens to equal a repr mangle (e.g. a type
        // literally named `Binary8`) becomes `Binary8#` and can NEVER collide with the repr
        // `Binary{8}` → `Binary8`. This keeps `mangle_ty`/`mangle_decl`/`item_key` injective across
        // the repr/data boundary, so two distinct instantiations never alias to one mangled name (no
        // silent drop — G2). The `#` appears only inside a composite name; a monomorphic data type is
        // still registered and referenced under its bare name (`mangle_ty_in_ty` clones a nullary
        // `Data` directly), so monomorphic passthrough is unaffected.
        Ty::Data(n, args) if args.is_empty() => format!("{n}#"),
        Ty::Data(n, args) => {
            let mut s = n.clone();
            for a in args {
                s.push('$');
                s.push_str(&mangle_ty(a));
            }
            s
        }
        // A `Ty::Var` must never reach mangling (mono refuses an undetermined parameter first); a
        // distinctive marker keeps a hypothetical leak observable rather than silently collidable.
        Ty::Var(v) => format!("VAR_{v}"),
        // RFC-0024 §4 / M-687: function-type parameters are defunctionalized in M-687.
        // A `Ty::Fn` reaching mangling before M-687 is a bug — use a distinctive, non-collidable
        // marker so the leak surfaces loudly (never silently — G2/VR-5).
        Ty::Fn(a, r) => format!("HOF_FN_{}__TO__{}", mangle_ty(a), mangle_ty(r)),
    }
}

/// The scalar tag used inside [`mangle_ty`] (`F16`/`BF16`/`F32`/`F64`).
fn scalar_tag(s: Scalar) -> &'static str {
    match s {
        Scalar::F16 => "F16",
        Scalar::Bf16 => "BF16",
        Scalar::F32 => "F32",
        Scalar::F64 => "F64",
    }
}

/// Mangle a declaration name (fn or data type) at concrete type arguments: `name` + `"$" + mangle_ty`
/// per argument. **Empty `targs` ⇒ the original name, byte-for-byte** — so monomorphic code and
/// non-generic programs are untouched.
pub(crate) fn mangle_decl(name: &str, targs: &[Ty]) -> String {
    if targs.is_empty() {
        return name.to_owned();
    }
    let mut s = name.to_owned();
    for t in targs {
        s.push('$');
        s.push_str(&mangle_ty(t));
    }
    s
}

/// Mangle a **constructor** name at its data type's concrete arguments — same scheme as
/// [`mangle_decl`] (empty `targs` ⇒ unchanged). Distinct instantiations get distinct ctor names so the
/// registry / [`Env::ctor`] key stays globally unique across mono'd data types.
pub(crate) fn mangle_ctor(name: &str, targs: &[Ty]) -> String {
    mangle_decl(name, targs)
}

/// Mangle a trait method to the direct monomorphic fn name `method$Trait$ForTy` — e.g.
/// `cmp$Cmp$Binary8`. The receiver is mangled with [`mangle_ty`]; the name encodes (method, trait,
/// receiver), which is the honest queryable identity of the resolved dispatch.
pub(crate) fn mangle_method(method: &str, trait_name: &str, for_ty: &Ty) -> String {
    format!("{method}${trait_name}${}", mangle_ty(for_ty))
}

/// Rewrite a concrete `Ty` so every applied data type becomes its **mangled-nullary** form
/// (`Data("List$Binary8", [])`), the shape `build_registry`/`field_spec` already handle. Primitive
/// reprs pass through unchanged.
fn mangle_ty_in_ty(t: &Ty) -> Ty {
    match t {
        Ty::Binary(_) | Ty::Ternary(_) | Ty::Dense(_, _) | Ty::Substrate(_) | Ty::Bytes => {
            t.clone()
        }
        // RFC-0032 D3: mangle the element type (it may carry a mono'd applied data type), keeping the
        // sequence structure; primitive element reprs pass through unchanged.
        Ty::Seq(elem, n) => Ty::Seq(Box::new(mangle_ty_in_ty(elem)), *n),
        Ty::Data(_, args) if args.is_empty() => t.clone(),
        Ty::Data(_, _) => Ty::Data(mangle_ty(t), vec![]),
        Ty::Var(v) => Ty::Var(v.clone()), // defended against earlier; pass through if it ever appears
        // RFC-0024 §4 / M-687: function types pass through un-mangled; the defunctionalization
        // rewrite in M-687 will eliminate them before any fn mangle/registry step.
        Ty::Fn(_, _) => t.clone(),
    }
}

/// Convert a concrete checked [`Ty`] back to a **source-named** surface [`TypeRef`] (no guarantee
/// index) — an applied data type keeps its original name and recurses into its arguments
/// (`List<Binary{8}>` → `Named("List", [Binary{8}])`). Used to thread an `expected` type into
/// re-inference (`infer_type`), which resolves names against the **source** env. (Contrast
/// [`ty_to_ref`], which produces the *mangled-nullary* output form for the emitted env.)
fn ty_to_source_ref(t: &Ty) -> TypeRef {
    let base = match t {
        Ty::Binary(Width::Lit(n)) => BaseType::Binary(WidthRef::Lit(*n)),
        Ty::Binary(Width::Var(v)) => BaseType::Binary(WidthRef::Name(v.clone())),
        Ty::Ternary(Width::Lit(m)) => BaseType::Ternary(WidthRef::Lit(*m)),
        Ty::Ternary(Width::Var(v)) => BaseType::Ternary(WidthRef::Name(v.clone())),
        Ty::Dense(d, s) => BaseType::Dense(*d, *s),
        Ty::Substrate(tag) => BaseType::Substrate(tag.clone()),
        // RFC-0032 D3/D4: round-trip the sequence/byte-string reprs to their surface forms.
        Ty::Seq(elem, n) => BaseType::Seq {
            elem: Box::new(ty_to_source_ref(elem)),
            len: *n,
        },
        Ty::Bytes => BaseType::Bytes,
        Ty::Data(n, args) => {
            BaseType::Named(n.clone(), args.iter().map(ty_to_source_ref).collect())
        }
        Ty::Var(v) => BaseType::Named(v.clone(), vec![]),
        // RFC-0024 §4 / M-687: function types round-trip as `BaseType::Fn`. Used only for re-inference
        // context threading; defunctionalization (M-687) rewrites them before any registry step.
        Ty::Fn(a, r) => BaseType::Fn(Box::new(ty_to_source_ref(a)), Box::new(ty_to_source_ref(r))),
    };
    TypeRef::unguaranteed(base)
}

/// Convert a concrete checked [`Ty`] back to a surface [`TypeRef`] (no guarantee index) so a rewritten
/// `FnDecl`/`Param`/`Ascribe` carries a concrete surface type. Mono erases type variables and bakes a
/// data type's arguments into its **mangled-nullary** name, so an applied `Ty::Data(_, args!=[])`
/// becomes the `Named` of its mangled name; a `Ty::Var` would be an internal error, surfaced as a
/// distinctive `Named` so a leak is never silent (rather than a panic).
fn ty_to_ref(t: &Ty) -> TypeRef {
    let base = match t {
        Ty::Binary(Width::Lit(n)) => BaseType::Binary(WidthRef::Lit(*n)),
        Ty::Binary(Width::Var(v)) => BaseType::Binary(WidthRef::Name(v.clone())),
        Ty::Ternary(Width::Lit(m)) => BaseType::Ternary(WidthRef::Lit(*m)),
        Ty::Ternary(Width::Var(v)) => BaseType::Ternary(WidthRef::Name(v.clone())),
        Ty::Dense(d, s) => BaseType::Dense(*d, *s),
        Ty::Substrate(tag) => BaseType::Substrate(tag.clone()),
        // RFC-0032 D3/D4: round-trip the sequence/byte-string reprs (the element type is mono'd to a
        // concrete surface form via the same `ty_to_ref`).
        Ty::Seq(elem, n) => BaseType::Seq {
            elem: Box::new(ty_to_ref(elem)),
            len: *n,
        },
        Ty::Bytes => BaseType::Bytes,
        // A mono'd data type is nullary (its arguments are baked into its mangled name).
        Ty::Data(n, args) if args.is_empty() => BaseType::Named(n.clone(), vec![]),
        Ty::Data(_, _) => BaseType::Named(mangle_ty(t), vec![]),
        Ty::Var(v) => BaseType::Named(format!("VAR_{v}"), vec![]),
        // RFC-0024 §4 / M-687: function types in rewritten fn-decl positions; defunctionalization
        // in M-687 will eliminate these. Preserve as `BaseType::Fn` so the AST stays structurally
        // sound (never a silent drop or panic — G2/VR-5).
        Ty::Fn(a, r) => BaseType::Fn(Box::new(ty_to_ref(a)), Box::new(ty_to_ref(r))),
    };
    TypeRef::unguaranteed(base)
}
