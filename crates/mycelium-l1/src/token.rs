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
    /// `use` — import (conventional).
    Use,
    /// `type` — data-type declaration.
    Type,
    /// `trait` — typeclass (conventional; `guild` was declined).
    Trait,
    /// `fn` — function.
    Fn,
    /// `matured` — promoted stable component (themed).
    Matured,
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
    /// `:`.
    Colon,
    /// `,`.
    Comma,
    /// `.`.
    Dot,
    /// `|`.
    Pipe,
    /// `=`.
    Eq,
    /// `->`.
    Arrow,
    /// `=>`.
    FatArrow,
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
        "use" => Tok::Use,
        "type" => Tok::Type,
        "trait" => Tok::Trait,
        "fn" => Tok::Fn,
        "matured" => Tok::Matured,
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
