//! Tokens and source spans for the L1 surface (RFC-0006; DN-02 vocabulary).

/// A 1-based source position, for never-silent parse diagnostics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pos {
    /// 1-based line.
    pub line: u32,
    /// 1-based column.
    pub col: u32,
}

impl core::fmt::Display for Pos {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}:{}", self.line, self.col)
    }
}

/// A lexical token. Keyword variants are the ratified DN-02 reserved words; an identifier that
/// matches a reserved word lexes as the keyword (so using e.g. `nodule` as a name is a parse
/// error, never a silent shadow — `reject/05`).
#[derive(Debug, Clone, PartialEq)]
pub enum Tok {
    // --- structural keywords (DN-02; nodule per DN-06) ---
    /// `nodule` — the basic static organizational unit (themed; DN-06, supersedes static `colony`).
    Nodule,
    /// `phylum` — the library-scale static grouping above `nodule` (DN-06). **Reserved, not yet
    /// active**: it lexes as a keyword (so it is never a silent identifier) but no construct consumes
    /// it yet, so it cannot open a program (RFC-0006 §4.3; its construct lands later).
    Phylum,
    /// `colony` — the **dynamic** runtime grouping of active `hypha` (DN-06 §2; RFC-0008 §4.7),
    /// reassigned from its former static meaning. **Reserved, not yet active** at the L1 surface:
    /// it lexes as a keyword (never a silent identifier) but no L1 construct consumes it; the
    /// realization lives in `mycelium-mlir::runtime` (M-357).
    Colony,

    // --- runtime-vocabulary reserved words (DN-03 §4; RFC-0008 §4.5) ---
    // All ten are **reserved, not yet active**: they lex as keywords (never silent identifiers,
    // G2) but no L1 construct consumes them. At item-declaration and expression position the parser
    // emits the explicit "reserved for the runtime model (RFC-0008), not yet active" diagnostic;
    // where an identifier is grammatically required (a fn/binder name, a program opener) the
    // standard "expected an identifier"/"expected a `nodule` header" error fires first. Either way
    // the word can NEVER be used as an identifier (G2 holds) — only the message differs.
    // Activation requires each construct's implementation RFC (RFC-0008 §4.6 R1/R2).
    /// `hypha` — concurrent execution unit (RFC-0008). **Reserved, not yet active.**
    Hypha,
    /// `fuse` — lawful state fusion / CRDT join (RFC-0008 RT6). **Reserved, not yet active.**
    Fuse,
    /// `mesh` — decentralized gossip/pub-sub overlay (RFC-0008 RT5). **Reserved, not yet active.**
    Mesh,
    /// `graft` — capability contract with infrastructure (RFC-0008 RT4). **Reserved, not yet active.**
    Graft,
    /// `cyst` — durable checkpoint / encystment into a dormant resumable form (RFC-0008 RT2). **Reserved, not yet active.**
    Cyst,
    /// `xloc` — explicit value movement / trans-locate (RFC-0008). **Reserved, not yet active.**
    Xloc,
    /// `forage` — adaptive placement policy (RFC-0008 RT3). **Reserved, not yet active.**
    Forage,
    /// `backbone` — priority transport path (RFC-0008 RT3). **Reserved, not yet active.**
    Backbone,
    /// `tier` — execution-mode switch (interpreted ↔ native, RFC-0008). **Reserved, not yet active.**
    Tier,
    /// `reclaim` — runtime-unit reclamation (stale units only, never memory — RFC-0008 RT7). **Reserved, not yet active.**
    Reclaim,

    /// `use` — import (conventional).
    Use,
    /// `pub` — the cross-nodule **export** marker on a top-level `fn`/`trait`/`type` (M-662; RFC-0006
    /// §4.3; conventional, Rust-like). A top-level item is private-to-nodule by default; `pub` exposes
    /// its name to the other nodules of the phylum. Reserved as a keyword so it can never be a silent
    /// identifier (G2). It precedes `fn`/`trait`/`type` (and `thaw fn`); a `pub` anywhere else is an
    /// explicit parse error.
    Pub,
    /// `type` — data-type declaration.
    Type,
    /// `trait` — typeclass (conventional; `guild` was declined).
    Trait,
    /// `impl` — trait-instance / inherent-method block (DN-03 §1; RFC-0019 §3.2 `impl Trait for T`;
    /// RFC-0007 §12). Reserved here (M-658) so it can never silently become an identifier (G2); the
    /// parser productions (`impl … for …` instances, `impl T { … }` inherent methods) land with the
    /// trait checker (M-659) and the surface-keyword work (M-664).
    Impl,
    /// `fn` — function.
    Fn,
    /// `matured` — **reserved keyword** (RFC-0017): maturation is now a scope/header attribute
    /// (`// @matured: true`), not a function modifier. Using it at item position is a parse error
    /// with a teaching diagnostic. Kept as a reserved keyword so it can never silently become an
    /// identifier (G2).
    Matured,
    /// `thaw` — de-maturation marker (RFC-0017 §4.3): `thaw fn f(…) -> T = …` keeps one
    /// definition interpreted inside an otherwise-matured scope.
    Thaw,
    /// `let`.
    Let,
    /// `in`.
    In,
    /// `if`.
    If,
    /// `then`.
    Then,
    /// `else`.
    Else,
    /// `match`.
    Match,
    /// `for` — bounded iteration sugar over structural recursion (RFC-0007 §4.8; r2).
    For,
    /// `swap` — the never-silent representation change (native corpus term).
    Swap,
    /// `default` — opens a nodule-scope ambient declaration (`default paradigm P`; RFC-0012 §4.2).
    Default,
    /// `paradigm` — the ambient granularity keyword (`default paradigm P` / `with paradigm P`).
    Paradigm,
    /// `with` — opens a block-scope ambient override (`with paradigm P { … }`; RFC-0012 §4.4).
    With,
    /// `wild` — the denied-by-default unsafe block (themed).
    Wild,
    /// `spore` — reconstruction-manifest construction (themed).
    Spore,
    /// `to` — the swap target label.
    To,
    /// `policy` — the swap policy label.
    Policy,

    // --- type keywords ---
    /// `Binary`.
    Binary,
    /// `Ternary`.
    Ternary,
    /// `Dense`.
    Dense,
    /// `VSA`.
    Vsa,
    /// `Substrate` — the affine external-resource kind (themed; LR-8).
    Substrate,
    /// `Sparse`.
    Sparse,
    /// A scalar kind keyword (`F16|BF16|F32|F64`).
    Scalar(ScalarTok),
    /// A guarantee-strength keyword (`Exact|Proven|Empirical|Declared`).
    Strength(StrengthTok),

    // --- identifiers & literals ---
    /// An identifier (incl. the lone `_` wildcard).
    Ident(String),
    /// A binary literal `0b…` (digits + `_` separators preserved verbatim).
    BinLit(String),
    /// A balanced-ternary literal `<…>` (the inner `+0-` string, MSB-first).
    TritLit(String),
    /// A non-negative decimal integer literal.
    Int(i64),

    // --- punctuation ---
    /// `(`.
    LParen,
    /// `)`.
    RParen,
    /// `{`.
    LBrace,
    /// `}`.
    RBrace,
    /// `[`.
    LBracket,
    /// `]`.
    RBracket,
    /// `<` (type-args open; trit literals are lexed whole).
    LAngle,
    /// `>`.
    RAngle,
    /// `@` — guarantee annotation.
    At,
    /// `@std-sys` — the explicit **nodule-header marker** for the audited FFI-floor context
    /// (RFC-0016 §8-Q6; LR-9/S6; M-661). Lexed as **one atomic token** (not `@` + `std-sys`, which
    /// would not lex — `-` is not an identifier char): the lexer recognizes `@` immediately followed
    /// by the literal `std-sys`. It can never collide with a `T @ Strength` guarantee annotation
    /// (that is `@` followed by a `Strength` keyword). A `wild` block is legal only inside a nodule
    /// whose header carries this marker (M-661); it is otherwise inert in v0. A bare `@std` (without
    /// the `-sys` tail) still lexes as `Tok::At` + `Tok::Ident("std")`, so this special case is
    /// maximally narrow.
    AtStdSys,
    /// `:`.
    Colon,
    /// `,`.
    Comma,
    /// `.`.
    Dot,
    /// `|`.
    Pipe,
    /// `+` — context-dependent (RFC-0025 / M-705): in a bounded type-parameter it is the
    /// trait-bound separator (`T: A + B`; RFC-0019 §4.1); at expression position it is the
    /// infix **add** operator (`a + b` desugars to `add(a, b)`). The two contexts never overlap
    /// (a bound list is inside `<…>`); outside both, the parser raises an explicit error (G2).
    Plus,
    /// `-` — context-dependent: the unary/binary **sub/neg** operator at expression position
    /// (`a - b` → `sub(a, b)`, `-a` → `neg(a)`; RFC-0025 / M-705). `->` lexes as [`Tok::Arrow`]
    /// (the function-type arrow), so a bare `-` is only ever the arithmetic operator.
    Minus,
    /// `*` — context-dependent (RFC-0025 / M-705): the **glob** marker as the tail of a wildcard
    /// import `use a.b.*` (M-662), or the infix **mul** operator at expression position
    /// (`a * b` → `mul(a, b)`). The two contexts never overlap; outside both it is an error (G2).
    Star,
    /// `/` — the infix **div** operator at expression position (`a / b` → `div(a, b)`;
    /// RFC-0025 / M-705). `//` opens a line comment (consumed as trivia), so a bare `/` is only
    /// ever the division operator.
    Slash,
    /// `%` — the infix **rem** operator at expression position (`a % b` → `rem(a, b)`;
    /// RFC-0025 / M-705).
    Percent,
    /// `^` — the infix bitwise-**xor** operator at expression position (`a ^ b` → `xor(a, b)`;
    /// RFC-0025 / M-705).
    Caret,
    /// `&` — the infix bitwise-**and** operator at expression position (`a & b` → `band(a, b)`;
    /// RFC-0025 / M-705).
    Amp,
    /// `&&` — the infix logical-**and** operator at expression position (`a && b` → `and(a, b)`;
    /// RFC-0025 / M-705).
    AmpAmp,
    /// `=`.
    Eq,
    /// `==` — the infix **eq** operator at expression position (`a == b` → `eq(a, b)`;
    /// RFC-0025 / M-705). `=` (single) stays the binder/definition glyph; `==` never collides
    /// with it (the binder is always a single `=`).
    EqEq,
    /// `->`.
    Arrow,
    /// `=>`.
    FatArrow,
    /// `!` — context-dependent: it opens the effect annotation `!{ … }` on a fn signature
    /// (RFC-0014 §3.4; M-660), and at expression position it is the unary bitwise-**not** operator
    /// (`!a` → `not(a)`; RFC-0025 / M-705). The effect-set use is only ever `!` immediately
    /// before `{` in a signature; outside both the parser raises an explicit error (G2). The
    /// effect *names* inside the braces stay ordinary identifiers.
    Bang,
    /// `!=` — the infix **ne** operator at expression position (`a != b` → `ne(a, b)`;
    /// RFC-0025 / M-705).
    BangEq,
    /// `||` — the infix logical-**or** operator at expression position (`a || b` → `or(a, b)`;
    /// RFC-0025 / M-705). `|` (single) stays the match-arm/pattern-alternation glyph and the
    /// bitwise-**or** operator (`a | b` → `bor(a, b)`); the two `|` uses never overlap (patterns
    /// are parsed in pattern position, the operator at expression position).
    PipePipe,
    /// End of input.
    Eof,
}

/// Scalar-kind keyword payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScalarTok {
    /// `F16`.
    F16,
    /// `BF16`.
    Bf16,
    /// `F32`.
    F32,
    /// `F64`.
    F64,
}

/// Guarantee-strength keyword payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StrengthTok {
    /// `Exact`.
    Exact,
    /// `Proven`.
    Proven,
    /// `Empirical`.
    Empirical,
    /// `Declared`.
    Declared,
}

/// A token with its starting position.
#[derive(Debug, Clone, PartialEq)]
pub struct Spanned {
    /// The token.
    pub tok: Tok,
    /// Where it starts.
    pub pos: Pos,
}

/// Resolve an identifier-shaped lexeme to its keyword token, or `None` if it is a plain identifier.
#[must_use]
pub fn keyword(word: &str) -> Option<Tok> {
    Some(match word {
        "nodule" => Tok::Nodule,
        // Reserved, not yet active (DN-06): they lex as keywords so they can never be silent
        // identifiers, but no L1 construct consumes them yet (a never-silent reservation, G2).
        "phylum" => Tok::Phylum,
        "colony" => Tok::Colony,
        // Reserved, not yet active (DN-03 §4; RFC-0008 §4.5): the runtime-vocabulary terms.
        // They lex as keywords (never silent identifiers, G2) but no L1 construct consumes them.
        "hypha" => Tok::Hypha,
        "fuse" => Tok::Fuse,
        "mesh" => Tok::Mesh,
        "graft" => Tok::Graft,
        "cyst" => Tok::Cyst,
        "xloc" => Tok::Xloc,
        "forage" => Tok::Forage,
        "backbone" => Tok::Backbone,
        "tier" => Tok::Tier,
        "reclaim" => Tok::Reclaim,
        "use" => Tok::Use,
        // `pub` — the M-662 cross-nodule export marker (reserved so it is never a silent identifier).
        "pub" => Tok::Pub,
        "type" => Tok::Type,
        "trait" => Tok::Trait,
        "impl" => Tok::Impl,
        "fn" => Tok::Fn,
        "matured" => Tok::Matured,
        "thaw" => Tok::Thaw,
        "let" => Tok::Let,
        "in" => Tok::In,
        "if" => Tok::If,
        "then" => Tok::Then,
        "else" => Tok::Else,
        "match" => Tok::Match,
        "for" => Tok::For,
        "swap" => Tok::Swap,
        "default" => Tok::Default,
        "paradigm" => Tok::Paradigm,
        "with" => Tok::With,
        "wild" => Tok::Wild,
        "spore" => Tok::Spore,
        "to" => Tok::To,
        "policy" => Tok::Policy,
        "Binary" => Tok::Binary,
        "Ternary" => Tok::Ternary,
        "Dense" => Tok::Dense,
        "VSA" => Tok::Vsa,
        "Substrate" => Tok::Substrate,
        "Sparse" => Tok::Sparse,
        "F16" => Tok::Scalar(ScalarTok::F16),
        "BF16" => Tok::Scalar(ScalarTok::Bf16),
        "F32" => Tok::Scalar(ScalarTok::F32),
        "F64" => Tok::Scalar(ScalarTok::F64),
        "Exact" => Tok::Strength(StrengthTok::Exact),
        "Proven" => Tok::Strength(StrengthTok::Proven),
        "Empirical" => Tok::Strength(StrengthTok::Empirical),
        "Declared" => Tok::Strength(StrengthTok::Declared),
        _ => return None,
    })
}
