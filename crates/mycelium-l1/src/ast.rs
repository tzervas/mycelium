//! The L1 surface AST (RFC-0006 §3; DN-02 vocabulary). v0 — the L1-facing core; it grows with the
//! L1 kernel-calculus RFC (typing judgments, elaboration to L0). Faithful to `mycelium.ebnf`.

/// A dotted path (`signals.demo`, `core.binary`); also a bare name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Path(pub Vec<String>);

/// A whole program: a `colony` header and its items.
#[derive(Debug, Clone, PartialEq)]
pub struct Colony {
    /// The colony's dotted name.
    pub path: Path,
    /// Top-level items.
    pub items: Vec<Item>,
}

/// A top-level item.
#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    /// `use path`.
    Use(Path),
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

/// A function definition. `matured` marks a promoted stable component (RFC-0004 §4; DN-02).
#[derive(Debug, Clone, PartialEq)]
pub struct FnDecl {
    /// Whether the definition is `matured` (AOT-promoted).
    pub matured: bool,
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    /// by construction (RFC-0007 §4.8; spelling provisional, KC-2-gated).
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
    /// `wild { body }` — the denied-by-default unsafe block (LR-9).
    Wild(Box<Expr>),
    /// `spore(value)` — reconstruction-manifest construction.
    Spore(Box<Expr>),
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
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    /// `0b…` (the digit/`_` string, verbatim).
    Bin(String),
    /// `<…>` (the inner `+0-` string, MSB-first).
    Trit(String),
    /// A decimal integer.
    Int(i64),
    /// A list literal `[e, …]`.
    List(Vec<Expr>),
}
