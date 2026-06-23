//! **Semantic-tokens provider** (M-730; `textDocument/semanticTokens/full`).
//!
//! Classifies the lexical token stream into LSP semantic-token types and emits the protocol's
//! relative-delta encoding. The legend is the **LSP layer of the ratified RFC-0026 §3.2 scope-name
//! table** (Accepted): `keyword`/`type`/`enumMember`/`number`/`operator`/`comment`/`variable` — the
//! standard LSP token types the table maps each lexer bucket to (e.g. the guarantee-strength bucket
//! → `enumMember`, the substrate/scalar types → `type`). The TextMate/tree-sitter layers of the same
//! table live in `tools/grammar/` (M-731), generated from the same lexer `keyword()`.
//!
//! Scope and honesty (`Declared`): classification is **purely lexical/token-kind** — every
//! identifier is `variable` because the lexer cannot tell a function name from a binding without
//! semantic context. A client should present these as syntax colors, never as type-aware
//! classification (VR-5). A lexer error yields an **empty** token stream (the parse error surfaces
//! on the diagnostics channel), never a fabricated highlight (G2).

use serde_json::{json, Value};

use mycelium_l1::token::Tok;

use crate::span::{lex_items, LexItem, LexKind};

/// The semantic-token **type legend**, in index order. The encoded stream's `tokenType` field is an
/// index into this list (LSP §`semanticTokens`). These are the standard LSP token types the ratified
/// RFC-0026 §3.2 table maps each lexer bucket to (the TextMate/tree-sitter names of the same table
/// live in `tools/grammar/`).
pub const TOKEN_TYPES: &[&str] = &[
    "keyword",    // 0 — reserved words (declaration + control + runtime vocabulary)
    "type",       // 1 — substrate/representation types (Binary/Ternary/Dense/VSA/…) + scalars
    "enumMember", // 2 — guarantee-strength lattice members (Exact/Proven/Empirical/Declared)
    "function", // 3 — (reserved in the legend; lexical classification cannot assign it — see note)
    "variable", // 4 — identifiers (lexical: every name, function or binding alike)
    "number",   // 5 — binary / balanced-ternary / integer literals
    "operator", // 6 — arithmetic/logical/annotation operators and arrows
    "comment",  // 7 — `//` line comments
];

/// LSP semantic-token type indices (into [`TOKEN_TYPES`]).
const T_KEYWORD: u32 = 0;
const T_TYPE: u32 = 1;
const T_ENUM_MEMBER: u32 = 2;
const T_VARIABLE: u32 = 4;
const T_NUMBER: u32 = 5;
const T_OPERATOR: u32 = 6;
const T_COMMENT: u32 = 7;

/// The `legend` advertised in the server's `semanticTokensProvider` capability: the type list above
/// and an empty modifier list (no modifiers are emitted — honest about scope).
#[must_use]
pub fn semantic_tokens_legend() -> Value {
    json!({
        "tokenTypes": TOKEN_TYPES,
        "tokenModifiers": [],
    })
}

/// Classify one lexical item to its [`TOKEN_TYPES`] index, or `None` for items that carry no
/// highlight (delimiters, `Eof`). Delimiters (`()[]{}`, `:` `,` `.` `<` `>`) are intentionally
/// unclassified: editors colour them via the grammar, not semantic tokens.
fn classify(kind: &LexKind) -> Option<u32> {
    let tok = match kind {
        LexKind::Comment => return Some(T_COMMENT),
        LexKind::Token(t) => t,
    };
    let idx = match tok {
        // Declaration + control + runtime-vocabulary keywords.
        Tok::Nodule
        | Tok::Phylum
        | Tok::Colony
        | Tok::Hypha
        | Tok::Fuse
        | Tok::Mesh
        | Tok::Graft
        | Tok::Cyst
        | Tok::Xloc
        | Tok::Forage
        | Tok::Backbone
        | Tok::Tier
        | Tok::Reclaim
        | Tok::Use
        | Tok::Pub
        | Tok::Type
        | Tok::Trait
        | Tok::Impl
        | Tok::Fn
        | Tok::Matured
        | Tok::Thaw
        | Tok::Let
        | Tok::In
        | Tok::If
        | Tok::Then
        | Tok::Else
        | Tok::Match
        | Tok::For
        | Tok::Swap
        | Tok::Default
        | Tok::Paradigm
        | Tok::With
        | Tok::Wild
        | Tok::Spore
        | Tok::To
        | Tok::Policy => T_KEYWORD,
        // Substrate / representation types and scalars.
        Tok::Binary
        | Tok::Ternary
        | Tok::Dense
        | Tok::Vsa
        | Tok::Substrate
        | Tok::Sparse
        | Tok::Scalar(_) => T_TYPE,
        // Guarantee-strength lattice members.
        Tok::Strength(_) => T_ENUM_MEMBER,
        // Identifiers — lexical only (no function/variable distinction available; see module note).
        Tok::Ident(_) => T_VARIABLE,
        // Literals.
        Tok::BinLit(_) | Tok::TritLit(_) | Tok::Int(_) => T_NUMBER,
        // Operators / annotations / arrows.
        Tok::Plus
        | Tok::Minus
        | Tok::Star
        | Tok::Slash
        | Tok::Percent
        | Tok::Caret
        | Tok::Amp
        | Tok::AmpAmp
        | Tok::Eq
        | Tok::EqEq
        | Tok::Arrow
        | Tok::FatArrow
        | Tok::Bang
        | Tok::BangEq
        | Tok::Pipe
        | Tok::PipePipe
        | Tok::At
        | Tok::AtStdSys => T_OPERATOR,
        // Delimiters and Eof carry no semantic-token highlight.
        _ => return None,
    };
    Some(idx)
}

/// Build the `textDocument/semanticTokens/full` result for `src`: the LSP relative-delta encoding
/// (`{ "data": [deltaLine, deltaStartChar, length, tokenType, tokenModifiers, …] }`).
///
/// The encoding is relative to the previous emitted token (LSP §`semanticTokens` "Integer Encoding"):
/// `deltaLine` is the line gap; `deltaStartChar` is the column gap when on the same line, else the
/// absolute 0-based start column; `length`/`tokenType` are the item's char length and legend index;
/// `tokenModifiers` is always `0`. Items are emitted in source order (already sorted by line/col).
/// Unclassified items (delimiters) are skipped. **Never-silent:** an un-lexable document yields
/// `{ "data": [] }` (the diagnostics channel reports the error), never a fabricated stream (G2).
#[must_use]
pub fn semantic_tokens_full(src: &str) -> Value {
    json!({ "data": encode(&lex_items(src)) })
}

/// The flat `u32` delta-stream for `items` (the body of [`semantic_tokens_full`], exposed for tests).
#[must_use]
pub(crate) fn encode(items: &[LexItem]) -> Vec<u32> {
    let mut data = Vec::new();
    let mut prev_line = 0u32;
    let mut prev_col0 = 0u32; // 0-based start column of the previous emitted token
    for it in items {
        let Some(ttype) = classify(&it.kind) else {
            continue;
        };
        let line0 = it.line.saturating_sub(1);
        let col0 = it.col.saturating_sub(1);
        let delta_line = line0 - prev_line;
        let delta_col = if delta_line == 0 {
            col0 - prev_col0
        } else {
            col0
        };
        data.extend_from_slice(&[delta_line, delta_col, it.len, ttype, 0]);
        prev_line = line0;
        prev_col0 = col0;
    }
    data
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legend_is_stable_and_indices_match() {
        // The encoded stream's tokenType indices must agree with the advertised legend order.
        let legend = semantic_tokens_legend();
        let types = legend["tokenTypes"].as_array().unwrap();
        assert_eq!(types[T_KEYWORD as usize], "keyword");
        assert_eq!(types[T_TYPE as usize], "type");
        assert_eq!(types[T_ENUM_MEMBER as usize], "enumMember");
        assert_eq!(types[T_NUMBER as usize], "number");
        assert_eq!(types[T_COMMENT as usize], "comment");
        assert!(legend["tokenModifiers"].as_array().unwrap().is_empty());
    }

    #[test]
    fn unlexable_source_is_empty_stream_not_a_panic() {
        // G2: never-silent — an un-lexable document highlights nothing rather than fabricating.
        let v = semantic_tokens_full("fn f() = §");
        assert_eq!(v["data"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn keyword_type_strength_and_number_are_classified() {
        // `fn` is a keyword, `Binary` a type, `Exact` an enumMember, `0b0` a number.
        let src = "fn f() -> Binary{8} @ Exact = 0b0\n";
        let data = encode(&lex_items(src));
        assert_eq!(data.len() % 5, 0, "every token is a 5-tuple");
        // Collect the tokenType column (index 3 of each 5-tuple).
        let ttypes: Vec<u32> = data.chunks(5).map(|c| c[3]).collect();
        assert!(ttypes.contains(&T_KEYWORD), "fn → keyword");
        assert!(ttypes.contains(&T_TYPE), "Binary → type");
        assert!(ttypes.contains(&T_ENUM_MEMBER), "Exact → enumMember");
        assert!(ttypes.contains(&T_NUMBER), "0b0 → number");
    }

    #[test]
    fn delta_encoding_is_relative_and_well_formed() {
        // Two lines: the first token of line 2 must encode a positive deltaLine and an absolute
        // (0-based) start column, not a column relative to the previous line.
        let src = "fn a()\nfn bb()\n";
        let data = encode(&lex_items(src));
        // First token: deltaLine 0, deltaStartChar 0 (the `fn` at col 1).
        assert_eq!(&data[0..2], &[0, 0]);
        // Find the first 5-tuple with a non-zero deltaLine: it begins line 2's `fn` at abs col 0.
        let line_break = data
            .chunks(5)
            .find(|c| c[0] == 1)
            .expect("a token starts a new line");
        assert_eq!(
            line_break[1], 0,
            "new-line token uses an absolute start column"
        );
    }
}
