//! The L1 surface AST (RFC-0006 §3; DN-02 vocabulary). v0 — the L1-facing core; it grows with the
//! L1 kernel-calculus RFC (typing judgments, elaboration to L0). Faithful to `mycelium.ebnf`.

/// A dotted path (`signals.demo`, `core.binary`); also a bare name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Path(pub Vec<String>);

/// A whole program: a `nodule` header and its items.
#[derive(Debug, Clone, PartialEq)]
pub struct Nodule {
    /// The nodule's dotted name.
    pub path: Path,
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
    /// `use path`.
    Use(Path),
    /// `default paradigm P` — the nodule-scope ambient (RFC-0012 §4.2). At most one per nodule; the
    /// outermost ambient frame. Consumed (stripped) by the resolution pass ([`crate::ambient`]).
    Default(Paradigm),
    /// A data-type declaration.
    Type(TypeDecl),
    /// A trait declaration.
    Trait(TraitDecl),
    /// A function definition.
    Fn(FnDecl),
}

/// `type Name<params> = Ctor | Ctor(field, …) | …` (LR-1).
#[derive(Debug, Clone, PartialEq)]
pub struct TypeDecl {
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

/// `trait Name<params> { fn … }` (LR-2; conventional term).
#[derive(Debug, Clone, PartialEq)]
pub struct TraitDecl {
    /// Trait name.
    pub name: String,
    /// Type parameters.
    pub params: Vec<String>,
    /// Required function signatures.
    pub sigs: Vec<FnSig>,
}

/// A function signature (shared by trait requirements and `fn` definitions).
#[derive(Debug, Clone, PartialEq)]
pub struct FnSig {
    /// Function name.
    pub name: String,
    /// Type parameters.
    pub params: Vec<String>,
    /// Value parameters.
    pub value_params: Vec<Param>,
    /// Result type.
    pub ret: TypeRef,
}

/// A function definition. `thaw` de-matures this def — keeps it interpreted inside a matured
/// scope; RFC-0017 §4.3. Maturation itself is a scope/header attribute, not a per-fn modifier.
#[derive(Debug, Clone, PartialEq)]
pub struct FnDecl {
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
    /// `Binary{width}`.
    Binary(u32),
    /// `Ternary{trits}`.
    Ternary(u32),
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
    /// A named type or type variable, with optional type arguments.
    Named(String, Vec<TypeRef>),
    /// A **paradigm-less repr** `{ <params> }` (RFC-0012 §4.2). Produced only by the parser; the
    /// resolution pass ([`crate::ambient`]) replaces it with the concrete paradigm from the
    /// enclosing ambient, or refuses (`UnresolvedAmbient`/`ParadigmShapeMismatch`). It never
    /// survives into the checker (defense-in-depth: a residual one is an explicit internal refusal).
    Ambient(AmbientParams),
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
    /// `wild { body }` — the denied-by-default unsafe block (LR-9).
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
    /// `<…>` (the inner `+0-` string, MSB-first).
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    #[test]
    fn typeref_unguaranteed_matches_field_form() {
        let base = BaseType::Binary(8);
        assert_eq!(
            TypeRef::unguaranteed(base.clone()),
            TypeRef {
                base,
                guarantee: None
            }
        );
    }

    #[test]
    fn typeref_with_guarantee_matches_field_form() {
        let base = BaseType::Ternary(3);
        assert_eq!(
            TypeRef::with_guarantee(base.clone(), Strength::Exact),
            TypeRef {
                base,
                guarantee: Some(Strength::Exact)
            }
        );
    }

    #[test]
    fn literal_ctors_match_variants() {
        assert_eq!(Literal::binary("1010"), Literal::Bin("1010".to_owned()));
        assert_eq!(Literal::ternary("+0-"), Literal::Trit("+0-".to_owned()));
        // `impl Into<String>` accepts both `&str` and `String`.
        assert_eq!(
            Literal::binary(String::from("11")),
            Literal::Bin("11".to_owned())
        );
    }

    fn hash_of<T: Hash>(t: &T) -> u64 {
        let mut h = DefaultHasher::new();
        t.hash(&mut h);
        h.finish()
    }

    #[test]
    fn scalar_and_strength_hash_is_consistent_with_eq() {
        // The new `Hash` derives must agree with `Eq` (equal values hash equal); enough to confirm
        // the derive is wired and usable as a map/set key.
        assert_eq!(hash_of(&Scalar::F32), hash_of(&Scalar::F32));
        assert_eq!(hash_of(&Strength::Proven), hash_of(&Strength::Proven));
        use std::collections::HashSet;
        let scalars: HashSet<Scalar> = [Scalar::F16, Scalar::Bf16, Scalar::F32, Scalar::F64]
            .into_iter()
            .collect();
        assert_eq!(scalars.len(), 4);
        let strengths: HashSet<Strength> = [
            Strength::Exact,
            Strength::Proven,
            Strength::Empirical,
            Strength::Declared,
        ]
        .into_iter()
        .collect();
        assert_eq!(strengths.len(), 4);
    }
}
