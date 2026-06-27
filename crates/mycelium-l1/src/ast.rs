//! The L1 surface AST (RFC-0006 §3; DN-02 vocabulary). v0 — the L1-facing core; it grows with the
//! L1 kernel-calculus RFC (typing judgments, elaboration to L0). Faithful to `mycelium.ebnf`.

/// A dotted path (`signals.demo`, `core.binary`); also a bare name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Path(pub Vec<String>);

/// A **phylum** — the library-scale static grouping above `nodule` (DN-06; RFC-0006 §4.3; M-662). A
/// phylum is a *grouping*, not a syntactic container: identity stays **per-nodule** (ADR-003), there
/// is no `phylum { … }` block. One source file holds an optional `phylum <path>` header followed by
/// one-or-more `nodule` blocks; it parses to this `Phylum`. A lone `nodule` with **no** `phylum`
/// header is a **phylum-of-one** (`path: None, nodules: [that_nodule]`) — every single-nodule program
/// is unchanged (the phylum is an additive layer; see [`crate::parse::parse`] vs
/// [`crate::parse::parse_phylum`]).
#[derive(Debug, Clone, PartialEq)]
pub struct Phylum {
    /// The phylum's dotted name from its `phylum <path>` header, or `None` for a header-less
    /// phylum-of-one (a bare single-nodule program).
    pub path: Option<Path>,
    /// The nodule(s) grouped by this phylum (≥ 1; the parser requires at least one `nodule` block).
    pub nodules: Vec<Nodule>,
}

impl Phylum {
    /// A **phylum-of-one** wrapping a single bare nodule (no `phylum` header). The additive bridge
    /// that lets every single-nodule program flow through the phylum-aware checker unchanged — a bare
    /// `nodule` *is* a phylum of one (M-662).
    #[must_use]
    pub fn of_one(nodule: Nodule) -> Self {
        Phylum {
            path: None,
            nodules: vec![nodule],
        }
    }
}

/// **Cross-nodule visibility** of a top-level item (M-662; RFC-0006 §4.3). Top-level `fn`/`trait`/
/// `type` are **private-to-nodule by default**; a `pub` marker exposes the name to **other** nodules
/// in the same phylum. *Intra*-nodule everything is visible regardless of `Vis` — `pub` gates **only**
/// cross-nodule visibility. (`impl`/`default`/`use` are never `pub`-gated.)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Vis {
    /// Private to its nodule (the default — no `pub` marker). Visible intra-nodule; invisible to
    /// other nodules of the phylum.
    Private,
    /// `pub` — exported to the other nodules of the phylum (cross-nodule visible).
    Pub,
}

impl Vis {
    /// Is this item exported to other nodules of the phylum (`pub`)?
    #[must_use]
    pub fn is_pub(self) -> bool {
        matches!(self, Vis::Pub)
    }
}

/// A `use` import target (`use a.b.Item` or the glob `use a.b.*`; M-662; RFC-0006 §4.3). A `use`
/// binds a name (or, for a glob, every `pub` name under a path) from another nodule of the phylum into
/// the local scope, keyed by the qualified name. Resolution is **never-silent** (G2): an unknown /
/// private / ambiguous import is an explicit `CheckError`, never a silent winner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsePath {
    /// The imported path. For a specific import it names the item (`a.b.Item`); for a glob it names
    /// the *prefix* whose `pub` names are imported (`a.b` from `use a.b.*`).
    pub path: Path,
    /// `true` for a glob `use a.b.*` (import all `pub` names under `path`); `false` for a specific
    /// `use a.b.Item`.
    pub glob: bool,
}

/// A whole program: a `nodule` header and its items.
#[derive(Debug, Clone, PartialEq)]
pub struct Nodule {
    /// The nodule's dotted name.
    pub path: Path,
    /// Whether the header carries the explicit **`@std-sys`** marker (`nodule std.sys.fs @std-sys`)
    /// — the audited FFI-floor context (RFC-0016 §8-Q6; LR-9/S6; M-661). This is a header
    /// **attribute**, *not* a naming convention: a `wild` block (the denied-by-default unsafe escape,
    /// LR-9) is legal **only** inside a nodule marked `@std-sys` — the checker hard-refuses a `wild`
    /// in any non-`@std-sys` nodule, never a silent escape (G2). The marker is parsed and threaded to
    /// the checker; it gates `wild` (and nothing else in v0).
    pub std_sys: bool,
    /// Top-level items.
    pub items: Vec<Item>,
}

/// A representation **paradigm** tag (RFC-0001 §4.2): the granularity of the RFC-0012 ambient. The
/// ambient supplies an *omitted paradigm*; widths/dims/dtypes/models stay explicit (the v0 scope).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Paradigm {
    /// `Binary`.
    Binary,
    /// `Ternary`.
    Ternary,
    /// `Dense`.
    Dense,
    /// `VSA`.
    Vsa,
}

impl core::fmt::Display for Paradigm {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(match self {
            Paradigm::Binary => "Binary",
            Paradigm::Ternary => "Ternary",
            Paradigm::Dense => "Dense",
            Paradigm::Vsa => "VSA",
        })
    }
}

/// The written params of a **paradigm-less repr** `{ … }` (RFC-0012 §4.2): the size/shape is still
/// written explicitly; only the paradigm is supplied by the enclosing ambient. The shape must fit
/// the ambient paradigm or resolution is an explicit `ParadigmShapeMismatch` (never a coerced guess).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AmbientParams {
    /// `{N}` — a single size (a `Binary` width or a `Ternary` trit count).
    Size(u32),
    /// `{N, scalar}` — a `Dense` shape.
    Dense(u32, Scalar),
    /// `{model, dim, sparsity}` — a `VSA` shape.
    Vsa {
        /// Model id.
        model: String,
        /// Dimension.
        dim: u32,
        /// Declared sparsity.
        sparsity: Sparsity,
    },
}

/// A top-level item.
#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    /// `use path` (specific) or `use path.*` (glob) — a cross-nodule import (M-662). Carries the
    /// import target ([`UsePath`]); a `use` is never `pub`-gated (importing is not re-exporting).
    Use(UsePath),
    /// `default paradigm P` — the nodule-scope ambient (RFC-0012 §4.2). At most one per nodule; the
    /// outermost ambient frame. Consumed (stripped) by the resolution pass ([`crate::ambient`]).
    Default(Paradigm),
    /// A data-type declaration.
    Type(TypeDecl),
    /// A trait declaration.
    Trait(TraitDecl),
    /// A trait-instance declaration `impl Trait<args> for T { fn … }` (RFC-0019 §4.1; LR-2).
    Impl(ImplDecl),
    /// A function definition.
    Fn(FnDecl),
}

/// `type Name<params> = Ctor | Ctor(field, …) | …` (LR-1). An optional leading `pub` marks the type
/// **exported** to other nodules of the phylum (M-662); absent ⇒ private-to-nodule.
#[derive(Debug, Clone, PartialEq)]
pub struct TypeDecl {
    /// Cross-nodule visibility (`pub` ⇒ [`Vis::Pub`], else [`Vis::Private`]; M-662). Intra-nodule
    /// the type is always visible — this gates only cross-nodule import.
    pub vis: Vis,
    /// Type name.
    pub name: String,
    /// Type parameters.
    pub params: Vec<String>,
    /// Constructors (≥ 1).
    pub ctors: Vec<Ctor>,
}

/// One constructor of a [`TypeDecl`].
#[derive(Debug, Clone, PartialEq)]
pub struct Ctor {
    /// Constructor name.
    pub name: String,
    /// Positional field types.
    pub fields: Vec<TypeRef>,
}

/// `trait Name<params> { fn … }` (LR-2; conventional term). `params` are **unbounded** type-variable
/// names in stage-1 (RFC-0019 §4.1 / RFC-0007 §12.1 — single-parameter traits; bounds on trait
/// parameters are a deferred refusal, never silently dropped).
#[derive(Debug, Clone, PartialEq)]
pub struct TraitDecl {
    /// Cross-nodule visibility (`pub` ⇒ [`Vis::Pub`]; M-662). Gates only cross-nodule import of the
    /// trait *name*; the orphan/coherence view is pub-blind (a trait is visible to coherence
    /// regardless of `Vis`).
    pub vis: Vis,
    /// Trait name.
    pub name: String,
    /// Type parameters (unbounded names; stage-1).
    pub params: Vec<String>,
    /// Required function signatures.
    pub sigs: Vec<FnSig>,
}

/// A trait-instance declaration `impl Trait<args> for T { fn … }` (RFC-0019 §4.1; RFC-0007 §12.1).
/// The methods are full function definitions (`fn name(params) -> ret = body`).
#[derive(Debug, Clone, PartialEq)]
pub struct ImplDecl {
    /// The trait being implemented.
    pub trait_name: String,
    /// The trait's type arguments (`impl Cmp<Binary{8}> for …` ⇒ `[Binary{8}]`). Concrete
    /// `TypeRef`s, not parameter names.
    pub trait_args: Vec<TypeRef>,
    /// The type the instance is for (`… for Binary{8}` ⇒ `Binary{8}`).
    pub for_ty: TypeRef,
    /// The provided method definitions.
    pub methods: Vec<FnDecl>,
}

/// A reference to a trait in a bound position — `Cmp` or `Cmp<Binary{8}>` (RFC-0019 §4.1 `bound`).
/// Appears only as an element of a [`TypeParam`]'s bounds (the dictionary site).
#[derive(Debug, Clone, PartialEq)]
pub struct TraitRef {
    /// The trait name.
    pub name: String,
    /// The trait's type arguments, if written (`Cmp<T>` ⇒ `[T]`; bare `Cmp` ⇒ `[]`).
    pub args: Vec<TypeRef>,
}

/// A width reference in a [`BaseType::Binary`] or [`BaseType::Ternary`] descriptor — either a
/// concrete literal (`Binary{8}`) or a width-parameter name (`Binary{N}` where `N` is a width
/// parameter of the enclosing `fn`; DN-42 / M-753 v1). Disambiguated from type parameters by the
/// width-slot (brace) context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WidthRef {
    /// A concrete width literal (e.g. `8` in `Binary{8}`).
    Lit(u32),
    /// A width-parameter name (e.g. `N` in `Binary{N}`; DN-42 / M-753 v1 — free functions only).
    Name(String),
}

impl core::fmt::Display for WidthRef {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            WidthRef::Lit(n) => write!(f, "{n}"),
            WidthRef::Name(v) => write!(f, "{v}"),
        }
    }
}

/// The classification of a type parameter declared in a `fn` signature's `<…>` list (DN-42 /
/// M-753 v1): either an ordinary **type** parameter (appears in type-slot positions) or a **width**
/// parameter (appears only in brace-width slots — `Binary{N}` or `Ternary{N}`). The distinction is
/// resolved post-parse by examining how the name is used in the value-parameter and return types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParamKind {
    /// Appears in type-slot positions (`Named(n, [])`) — the classic type parameter.
    Type,
    /// Appears only in width-slot positions (`Binary{N}` or `Ternary{N}`) — a width parameter.
    /// Cannot carry trait bounds in v1 (explicit refusal; DN-42 §7).
    Width,
}

/// A (possibly **bounded**) type parameter on a **function** — `T` or `T: Cmp + Ord<T>` (RFC-0019
/// §4.1 `type_param`). Bounds live **only** on function type-params (the dictionary site); data/trait
/// type-params stay unbounded names in stage-1 ([`TypeDecl::params`] / [`TraitDecl::params`]).
#[derive(Debug, Clone, PartialEq)]
pub struct TypeParam {
    /// The parameter name.
    pub name: String,
    /// Whether this is a **type** parameter or a **width** parameter (DN-42 / M-753 v1). Set by a
    /// post-parse classification pass in `parse_sig_tail` (`ParamKind::Type` is the default for
    /// every v0 program — purely additive).
    pub kind: ParamKind,
    /// Its trait bounds (empty for an unbounded parameter — the §11 identity case). Width params
    /// cannot carry bounds in v1 (explicit refusal at the classification pass; DN-42 §7).
    pub bounds: Vec<TraitRef>,
}

/// A function signature (shared by trait requirements and `fn` definitions).
#[derive(Debug, Clone, PartialEq)]
pub struct FnSig {
    /// Function name.
    pub name: String,
    /// Type parameters, possibly **bounded** (RFC-0019 §4.1). An unbounded `T` is `TypeParam { name:
    /// "T", bounds: [] }` — the §11 identity.
    pub params: Vec<TypeParam>,
    /// Value parameters.
    pub value_params: Vec<Param>,
    /// Result type.
    pub ret: TypeRef,
    /// The **declared effect set** (RFC-0014 §3.4/§4.5 I3; M-660) — the `!{eff1, eff2}` annotation
    /// after the return type, as surface effect **names** in source order. **Empty = pure** (an
    /// unannotated `fn`; RFC-0014 I5 default-tightly-scoped). The names are plain identifiers (NOT
    /// reserved words): the closed kernel effect kinds (`retry|alloc|io|cascade|time`) plus
    /// user-declared `Named` effects (RFC-0014 §4.5). Stored as `Vec<String>` — the surface names the
    /// effect-coverage checker compares by string (the v0 mechanism; mapping a name to
    /// `mycelium_interp::budget::EffectKind` is the *runtime* ledger's concern — M-353 — out of the L1
    /// frontend's scope). These are checker metadata only: effects lower to **no** L0 node (KC-3).
    pub effects: Vec<String>,
}

impl FnSig {
    /// The **names** of this signature's type parameters (dropping any bounds and width params) —
    /// the form the checker's `tyvars` scope and the §11 generic machinery consume (each name
    /// resolves to a `Ty::Var`). Additive helper so callers need not reach through each
    /// [`TypeParam`] (DRY / Law of Demeter); the bounds are read separately where
    /// instance-satisfiability is checked.
    #[must_use]
    pub fn param_names(&self) -> Vec<String> {
        self.params
            .iter()
            .filter(|p| p.kind == ParamKind::Type)
            .map(|p| p.name.clone())
            .collect()
    }

    /// The **names** of this signature's width parameters (DN-42 / M-753 v1). Empty for v0 programs.
    #[must_use]
    pub fn width_param_names(&self) -> Vec<String> {
        self.params
            .iter()
            .filter(|p| p.kind == ParamKind::Width)
            .map(|p| p.name.clone())
            .collect()
    }
}

/// A function definition. `thaw` de-matures this def — keeps it interpreted inside a matured
/// scope; RFC-0017 §4.3. Maturation itself is a scope/header attribute, not a per-fn modifier.
#[derive(Debug, Clone, PartialEq)]
pub struct FnDecl {
    /// Cross-nodule visibility (`pub` ⇒ [`Vis::Pub`]; M-662). A top-level `pub fn` is exported to the
    /// phylum's other nodules; absent ⇒ private-to-nodule. (An `impl` method's `FnDecl` is never
    /// `pub`-gated — `impl`/`default` are not part of the `pub` namespace; its `vis` stays
    /// [`Vis::Private`] and is ignored.)
    pub vis: Vis,
    /// `thaw` de-matures this def — keeps it interpreted inside a matured scope; RFC-0017 §4.3.
    pub thaw: bool,
    /// Its signature.
    pub sig: FnSig,
    /// Its body expression.
    pub body: Expr,
}

/// A value parameter `name: type`.
#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    /// Parameter name.
    pub name: String,
    /// Parameter type.
    pub ty: TypeRef,
}

/// A type with an optional guarantee-strength index (`T @ Exact`; LR-6).
#[derive(Debug, Clone, PartialEq)]
pub struct TypeRef {
    /// The underlying type.
    pub base: BaseType,
    /// The honesty index, if annotated.
    pub guarantee: Option<Strength>,
}

impl TypeRef {
    /// A type with **no** written guarantee index — the surface `T` form (the index is then
    /// supplied by the checked context, never silently defaulted; RFC-0007 §4.3). Equivalent to the
    /// `TypeRef { base, guarantee: None }` literal the parser builds; offered as a named, additive
    /// constructor so callers need not reach through the struct fields (Law of Demeter).
    #[must_use]
    pub fn unguaranteed(base: BaseType) -> Self {
        TypeRef {
            base,
            guarantee: None,
        }
    }

    /// A type carrying an explicit guarantee-strength index — the surface `T @ g` form. Equivalent
    /// to `TypeRef { base, guarantee: Some(g) }`. Additive convenience; it only *records* the index,
    /// it does not check it (that stays the typechecker/evaluator's never-silent job — VR-5).
    #[must_use]
    pub fn with_guarantee(base: BaseType, guarantee: Strength) -> Self {
        TypeRef {
            base,
            guarantee: Some(guarantee),
        }
    }
}

/// A base (un-annotated) type.
#[derive(Debug, Clone, PartialEq)]
pub enum BaseType {
    /// `Binary{width}` — the width slot may be a concrete literal or a width-parameter name
    /// (`Binary{N}`; DN-42 / M-753 v1). Use [`WidthRef::Lit`] for concrete programs; [`WidthRef::Name`]
    /// for width-generic signatures (resolved to [`crate::checkty::Width::Var`] during checking).
    Binary(WidthRef),
    /// `Ternary{trits}` — same width-slot generalization as [`BaseType::Binary`] (DN-42 / M-753 v1).
    Ternary(WidthRef),
    /// `Dense{dim, scalar}`.
    Dense(u32, Scalar),
    /// `VSA{model, dim, sparsity}`.
    Vsa {
        /// Model id.
        model: String,
        /// Dimension.
        dim: u32,
        /// Declared sparsity.
        sparsity: Sparsity,
    },
    /// `Substrate{name}` — an affine external resource (LR-8).
    Substrate(String),
    /// `Seq{elem, len}` — a first-class indexed homogeneous sequence (RFC-0032 D3; M-749). The
    /// descriptor carries the element type (`elem`, any [`TypeRef`]) and the declared element count
    /// (`len`, a `u32` literal). The surface literal `[e1, …]` constructs one; the type checks its
    /// elements are homogeneous and counts match `len` (never-silent — G2).
    Seq {
        /// The (boxed) element type.
        elem: Box<TypeRef>,
        /// The declared element count.
        len: u32,
    },
    /// `Bytes` — a first-class byte string (RFC-0032 D4; M-750). A **nullary** repr keyword (no
    /// descriptor). The surface literal `0x…` constructs one.
    Bytes,
    /// A named type or type variable, with optional type arguments.
    Named(String, Vec<TypeRef>),
    /// A **paradigm-less repr** `{ <params> }` (RFC-0012 §4.2). Produced only by the parser; the
    /// resolution pass ([`crate::ambient`]) replaces it with the concrete paradigm from the
    /// enclosing ambient, or refuses (`UnresolvedAmbient`/`ParadigmShapeMismatch`). It never
    /// survives into the checker (defense-in-depth: a residual one is an explicit internal refusal).
    Ambient(AmbientParams),
    /// **Function type** `A -> B` (RFC-0024 §3, HOF stage 1 — surface only). Single-argument
    /// v1; right-associative; `@` binds tighter than `->` (so `A @ Exact -> B` parses as
    /// `(A @ Exact) -> B`). The checker and mono are responsible for defunctionalization
    /// (M-686/M-687); this variant does **not** survive past the checker in v1 (deferred —
    /// multi-argument `(A, B) -> C` is not yet supported and is a never-silent refusal at the
    /// parser).
    Fn(Box<TypeRef>, Box<TypeRef>),
}

/// Declared sparsity of a VSA type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Sparsity {
    /// Dense.
    Dense,
    /// `Sparse{max_active}`.
    Sparse(u32),
}

/// A scalar element kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Scalar {
    /// `F16`.
    F16,
    /// `BF16`.
    Bf16,
    /// `F32`.
    F32,
    /// `F64`.
    F64,
}

/// A guarantee-lattice strength.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Strength {
    /// `Exact`.
    Exact,
    /// `Proven`.
    Proven,
    /// `Empirical`.
    Empirical,
    /// `Declared`.
    Declared,
}

impl Strength {
    /// The **trust rank** on the integrity lattice `Exact ⊐ Proven ⊐ Empirical ⊐ Declared`
    /// (RFC-0018 §4.1; Biba 1977, T3.2). Higher = more trusted: `Exact = 3 … Declared = 0`. This is
    /// the *only* place the chain's order is encoded; [`Self::meet`] and [`Self::satisfies`] derive
    /// from it (DRY). It is **not** a guarantee strength itself — just the comparison key.
    #[must_use]
    pub fn rank(self) -> u8 {
        match self {
            Strength::Declared => 0,
            Strength::Empirical => 1,
            Strength::Proven => 2,
            Strength::Exact => 3,
        }
    }

    /// The **meet** `g₁ ∧ g₂` — the *weaker* (less trusted) of the two grades (RFC-0018 §4.1: the
    /// greatest lower bound in the trust order). This is composition's pessimistic rule: a value
    /// built from mixed-grade parts carries the weakest (`Proven ∧ Empirical = Empirical`). The meet
    /// can only ever **lower** a grade — the structural reason grade composition is honest (it never
    /// claims more than the least-trusted input supports — VR-5).
    #[must_use]
    pub fn meet(self, other: Strength) -> Strength {
        if self.rank() <= other.rank() {
            self
        } else {
            other
        }
    }

    /// `self ⊒ demand` — is `self` **at least as trusted** as `demand`? The honesty rule as a
    /// comparison (RFC-0018 §4.3 G-Sub / G-App / G-Weaken): an argument may be passed to a parameter,
    /// a body may satisfy a return, and an annotation may weaken, **only** when the value's actual
    /// grade is `⊒` the demanded one. A `@ Empirical` value does **not** satisfy an `@ Exact` demand.
    #[must_use]
    pub fn satisfies(self, demand: Strength) -> bool {
        self.rank() >= demand.rank()
    }
}

/// An expression.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// `let name (: ty)? = bound in body`.
    Let {
        /// Bound name.
        name: String,
        /// Optional ascription.
        ty: Option<TypeRef>,
        /// The bound expression.
        bound: Box<Expr>,
        /// The body.
        body: Box<Expr>,
    },
    /// `if cond then conseq else alt`.
    If {
        /// Condition.
        cond: Box<Expr>,
        /// Then-branch.
        conseq: Box<Expr>,
        /// Else-branch.
        alt: Box<Expr>,
    },
    /// `match scrutinee { arm, … }`.
    Match {
        /// The scrutinee.
        scrutinee: Box<Expr>,
        /// The arms (≥ 1).
        arms: Vec<Arm>,
    },
    /// `for x in xs, acc = init => body` — bounded iteration over a linearly recursive data
    /// value; elaboration-defined sugar for a synthesized structurally-recursive fold, `Total`
    /// by construction (RFC-0007 §4.8; spelling adopted at r3).
    For {
        /// The element binder.
        x: String,
        /// The iterated (spine) value.
        xs: Box<Expr>,
        /// The accumulator binder.
        acc: String,
        /// The initial accumulator.
        init: Box<Expr>,
        /// The per-element body (yields the next accumulator).
        body: Box<Expr>,
    },
    /// `swap(value, to: target, policy: path)` — never-silent (S1).
    Swap {
        /// The value being converted.
        value: Box<Expr>,
        /// The target representation.
        target: TypeRef,
        /// The policy reference.
        policy: Path,
    },
    /// `with paradigm P { body }` — a block establishing a nested ambient over `body` (RFC-0012
    /// §4.4). It is **not** a conversion (it inserts no `Swap`, I1): a value crossing the boundary
    /// whose paradigm differs needs an explicit `swap`, and an unbridged edge is a never-silent
    /// `MissingConversion` refusal. The resolution pass strips it to just `body` after filling the
    /// interior tags; it never survives into the checker.
    WithParadigm {
        /// The interior ambient paradigm.
        paradigm: Paradigm,
        /// The block body.
        body: Box<Expr>,
    },
    /// `wild { body }` — the **audited FFI floor** (LR-9/S6; ADR-014; M-661). Parsed anywhere an
    /// expression may appear; its *legality* is a checker gate (`crate::checkty`): legal **only**
    /// inside a `@std-sys` nodule ([`Nodule::std_sys`]) whose enclosing `fn` declares the `ffi`
    /// effect — else a hard refusal (never silent — G2). The boxed `body` is the trusted/opaque FFI
    /// escape: not recursively type-checked (audited, not verified — VR-5), kept verbatim. Execution
    /// is staged (no FFI host in v0 → an elaboration `Residual`).
    Wild(Box<Expr>),
    /// `spore(value)` — reconstruction-manifest construction.
    Spore(Box<Expr>),
    /// `colony { hypha e1, hypha e2, … }` — the **structured-concurrency scope** (RFC-0008 §4.7;
    /// DN-06 §1.3): a dynamic runtime grouping of cooperating `hypha`. The block body is a
    /// **non-empty** list of `hypha` spawns; the colony does not exit until every child has joined
    /// (RT7 — "an orphan hypha is not expressible"). Deterministic R1 fragment only (RFC-0008 §4.6
    /// R1): the **reference semantics is the spawn-order sequentialization** (RT2), so the colony's
    /// observable is its children run in order, never a scheduler-dependent value.
    ///
    /// Honesty (Declared): this is the L1 *surface* for the RFC-0008 §4.7 model. It lowers two ways
    /// off **one** sequential trusted base (the L0 Core IR has **no** concurrency node — KC-3;
    /// RFC-0008 §4.2):
    /// - [`crate::elab::elaborate`] → the **RT2 spawn-order sequentialization** (a `Let` chain): the
    ///   deterministic *reference* the interpreter and AOT both run, and the oracle the concurrent
    ///   run is validated against;
    /// - [`crate::elab::elaborate_colony`] → one **closed L0 program per hypha**, which the
    ///   `mycelium-mlir::runtime` executor (`Scope`/`Colony`/`Task`, structured fork/join, M-357)
    ///   runs as **concurrent tasks** (`mycelium_mlir::run_colony`), validating the concurrent
    ///   observable **equals** the sequential reference (RT2) — an inequality is an explicit
    ///   divergence, never a silent race (G2/RT4).
    ///
    /// Both paths refuse outside the evaluation-complete fragment with a never-silent
    /// [`crate::elab::ElabError::Residual`] (G2), never a fabricated accept.
    Colony(Vec<Hypha>),
    /// `lambda(params) => body` — an anonymous-function expression (RFC-0037 D5). The grammar + AST
    /// node land with the RFC-0037 grammar epic; **full closure semantics (environment capture,
    /// partial application, dynamic fn-flow) are deferred to M-704 / RFC-0024 §5** — the checker
    /// ([`crate::checkty`]) and elaborator ([`crate::elab`]) emit a **never-silent `Residual`** for a
    /// lambda until then (G2), so the surface parses but does not yet evaluate. v0 params are typed
    /// (`name: type`); inferred-type params are part of the M-704 work.
    Lambda {
        /// The (typed) value parameters.
        params: Vec<Param>,
        /// The body expression.
        body: Box<Expr>,
    },
    /// A function/constructor application `head(args)` (possibly nested), or a bare head.
    App {
        /// The applied head.
        head: Box<Expr>,
        /// The arguments.
        args: Vec<Expr>,
    },
    /// A path/variable reference.
    Path(Path),
    /// A literal.
    Lit(Literal),
    /// `expr : type` ascription.
    Ascribe(Box<Expr>, TypeRef),
}

/// A `match` arm.
#[derive(Debug, Clone, PartialEq)]
pub struct Arm {
    /// The pattern.
    pub pattern: Pattern,
    /// The arm body.
    pub body: Expr,
}

/// One `hypha <expr>` spawn inside a [`Expr::Colony`] block — a single concurrent execution unit
/// (RFC-0008 §4.5: "structurally-scoped concurrent computation over immutable values"; RT1/RT2/RT7).
/// A `hypha` is **only** expressible inside a `colony` (RT7 — structured lifetimes; "an orphan
/// hypha is not expressible"), so it is a child of [`Expr::Colony`] rather than a free [`Expr`]
/// variant. Its body runs the deterministic R1 fragment (RFC-0008 §4.6 R1); its value is the value
/// the computation produces (RT1: values move, state is never shared).
#[derive(Debug, Clone, PartialEq)]
pub struct Hypha {
    /// The spawned computation (an application/expression over immutable values).
    pub body: Expr,
}

/// A pattern.
#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    /// `_`.
    Wildcard,
    /// A literal pattern.
    Lit(Literal),
    /// A constructor pattern `Name(sub, …)`.
    Ctor(String, Vec<Pattern>),
    /// A bare identifier (binder or nullary constructor — resolved later).
    Ident(String),
}

/// A literal value.
///
/// `#[non_exhaustive]`: the bare/ambient family has grown once already (`AmbientInt` arrived with
/// RFC-0012) and may grow again, so an *external* crate must keep a `_` arm — additive to the
/// public surface, never a removal (the attribute is added, no variant changes). In-crate matches
/// are unaffected by the attribute and stay exhaustive; no in-workspace caller matches `Literal`
/// today (M-642 survey), so nothing breaks.
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum Literal {
    /// `0b…` (the digit/`_` string, verbatim).
    Bin(String),
    /// `0t…` (the inner `+0-` string, MSB-first; RFC-0037 D4 — the former `<…>` form is retired).
    Trit(String),
    /// A decimal integer.
    Int(i64),
    /// A **bare decimal under an ambient** (RFC-0012 §4.3): the paradigm is supplied by the
    /// enclosing ambient; the *width* comes from the checked context. Produced only by the
    /// resolution pass ([`crate::ambient`]) from an [`Literal::Int`]; the checker resolves the
    /// width and rewrites it to a concrete [`Literal::Bin`]/[`Literal::Trit`], or refuses with an
    /// explicit `UnresolvedWidth` (never a built-in default). It never reaches elaboration.
    AmbientInt(Paradigm, i64),
    /// A list literal `[e, …]`.
    List(Vec<Expr>),
    /// A byte-string literal `0x…` (the inner hex/`_` string, verbatim; RFC-0032 D4, M-750).
    /// Elaborates to a [`mycelium_core::Repr::Bytes`] value. The lexer is the never-silent gate
    /// that only ever builds an even-hex-digit, non-empty string (G2), so the stored text is valid.
    Bytes(String),
}

impl Literal {
    /// A binary literal from its verbatim digit/`_` string (the `…` of `0b…`). Additive alias for
    /// [`Literal::Bin`]; like the variant it stores the string **verbatim** — it does not validate
    /// the digits (the lexer is the never-silent gate that only ever builds well-formed ones).
    #[must_use]
    pub fn binary(digits: impl Into<String>) -> Self {
        Literal::Bin(digits.into())
    }

    /// A ternary literal from its verbatim `+0-` string, MSB-first (the inner text of `<…>`).
    /// Additive alias for [`Literal::Trit`]; stores the string verbatim, no validation (see
    /// [`Literal::binary`]).
    #[must_use]
    pub fn ternary(trits: impl Into<String>) -> Self {
        Literal::Trit(trits.into())
    }
}
