//! The L1 lexer (RFC-0006; DN-02). Hand-written, no dependencies (house style). Produces a token
//! stream or an explicit [`ParseError`] — never a silent skip of an unrecognized character.
//!
//! The one subtlety is `<`: it opens both a balanced-ternary literal (`<+0->`) and a type-argument
//! list (`List<Ternary{6}>`). They are disambiguated by one character of lookahead — a `<`
//! immediately followed by a trit glyph (`+`, `-`, `0`) is a ternary literal (scanned whole),
//! anything else is the [`Tok::LAngle`] punctuation. A literal with a non-trit glyph before its
//! closing `>` is an explicit error (`reject/04`), never a silent truncation.
//!
//! ## Comment capture
//!
//! The public entry [`lex_with_comments`] returns the same token stream as [`lex`] **plus** an
//! ordered [`Vec<Comment>`] containing every `//` comment in the source.  The plain [`lex`]
//! function is behavior-identical to before (comments still do not appear in the token stream).
//!
//! ### What `Comment::text` stores
//!
//! The `text` field holds the **full lexeme from `//` through (but not including) the terminating
//! `\n`** (or end-of-file).  The leading `//` is included verbatim; there is no trailing newline
//! or carriage-return in `text`.  This makes round-trip re-emission straightforward: write
//! `comment.text`, then a newline.  No content is omitted and no content is altered (G2 —
//! never-silent capture).

use crate::error::ParseError;
use crate::token::{keyword, Pos, Spanned, Tok};

/// A captured `//` line comment, produced by [`lex_with_comments`].
///
/// Guarantee: `Empirical` — the `text`/`line`/`col`/`trailing` fields are populated by
/// the lexer's own line/column counters, which are validated by the unit tests in this module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Comment {
    /// The full lexeme from `//` through end-of-line content (verbatim, not including the
    /// terminating newline or carriage-return).  The leading `//` is always present.
    /// Example: for the source line `  // nodule: foo`, `text` is `"// nodule: foo"`.
    pub text: String,
    /// 1-based line number of the `//` opener.
    pub line: u32,
    /// 1-based column number of the `//` opener.
    pub col: u32,
    /// `true` iff at least one non-comment token was emitted on **this same source line**
    /// before the comment was encountered — i.e. this is an end-of-line (trailing) comment
    /// like `x => y  // why`.  `false` for a full-line (leading) comment.
    pub trailing: bool,
}

struct Lexer {
    chars: Vec<char>,
    i: usize,
    line: u32,
    col: u32,
    /// Comments collected so far, in source order.  Populated during `skip_trivia` calls.
    comments: Vec<Comment>,
    /// The source line on which the most-recently-emitted non-comment token started, or `0`
    /// if no token has been emitted yet.  Used to compute [`Comment::trailing`].
    last_token_line: u32,
}

/// Tokenize `src` into a [`Spanned`] stream terminated by [`Tok::Eof`].
///
/// Comments are **discarded** (behavior-identical to the original implementation).  Use
/// [`lex_with_comments`] to obtain the comment side-table alongside the same token stream.
pub fn lex(src: &str) -> Result<Vec<Spanned>, ParseError> {
    lex_with_comments(src).map(|(toks, _comments)| toks)
}

/// Tokenize `src`, returning the [`Spanned`] token stream **and** an ordered [`Vec<Comment>`]
/// with every `//` comment in source order.
///
/// The token stream is byte-identical to what [`lex`] returns; no comment appears in the tokens.
/// No comment is silently dropped — every `//`-to-EOL run is captured (G2).
///
/// # Errors
///
/// Returns a [`ParseError`] on any lexically invalid input (e.g. an unrecognized character or a
/// malformed ternary literal) — the same conditions under which [`lex`] returns `Err`.
pub fn lex_with_comments(src: &str) -> Result<(Vec<Spanned>, Vec<Comment>), ParseError> {
    let mut lx = Lexer {
        chars: src.chars().collect(),
        i: 0,
        line: 1,
        col: 1,
        comments: Vec::new(),
        last_token_line: 0,
    };
    let toks = lx.run()?;
    Ok((toks, lx.comments))
}

impl Lexer {
    fn pos(&self) -> Pos {
        Pos {
            line: self.line,
            col: self.col,
        }
    }

    /// Record a token emission so that subsequent comments on the same line are marked trailing.
    fn note_token_at(&mut self, line: u32) {
        self.last_token_line = line;
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.i).copied()
    }

    fn peek2(&self) -> Option<char> {
        self.chars.get(self.i + 1).copied()
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.chars.get(self.i).copied()?;
        self.i += 1;
        if c == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        Some(c)
    }

    fn run(&mut self) -> Result<Vec<Spanned>, ParseError> {
        let mut out = Vec::new();
        loop {
            self.skip_trivia();
            let pos = self.pos();
            let Some(c) = self.peek() else {
                // Eof does not update last_token_line — it carries no source line of its own.
                out.push(Spanned { tok: Tok::Eof, pos });
                return Ok(out);
            };
            let tok = match c {
                '(' => self.single(Tok::LParen),
                ')' => self.single(Tok::RParen),
                '{' => self.single(Tok::LBrace),
                '}' => self.single(Tok::RBrace),
                '[' => self.single(Tok::LBracket),
                ']' => self.single(Tok::RBracket),
                '>' => self.single(Tok::RAngle),
                // `@` is the guarantee-annotation glyph (`T @ Exact`), but `@std-sys` is the atomic
                // nodule-header FFI-floor marker (M-661): `-` is not an identifier char, so `@std-sys`
                // could never lex as `@` + an identifier — it must be recognized whole here. Only the
                // exact `@std-sys` is special; any other `@…` stays the bare `Tok::At`.
                '@' => self.lex_at(),
                ':' => self.single(Tok::Colon),
                ',' => self.single(Tok::Comma),
                '.' => self.single(Tok::Dot),
                '|' => self.single(Tok::Pipe),
                // `!` opens the effect annotation `!{ … }` (RFC-0014 §3.4; M-660). Single-char token;
                // the effect names inside stay identifiers. v0 has no other `!` use, so the parser
                // accepts it only before an effect set and errors otherwise (never a silent accept).
                '!' => self.single(Tok::Bang),
                // `+` is the trait-bound separator (`T: A + B`; RFC-0019 §4.1). It is also a trit
                // glyph, but a trit literal is only ever scanned *whole* from an opening `<` (in
                // `lex_angle_or_trit`), so a `+` reaching here is always the bound separator token.
                '+' => self.single(Tok::Plus),
                // `*` is the glob marker of a wildcard import `use a.b.*` (M-662). v0 has no other
                // `*` use, so the parser accepts it only as a glob-`use` tail and errors otherwise.
                '*' => self.single(Tok::Star),
                '<' => self.lex_angle_or_trit(pos)?,
                '=' => self.lex_eq(),
                '-' => self.lex_dash(pos)?,
                '0' if self.peek2() == Some('b') => self.lex_binary(),
                c if c.is_ascii_digit() => self.lex_int(pos)?,
                c if is_ident_start(c) => self.lex_ident(),
                other => {
                    return Err(ParseError::new(
                        pos,
                        format!("unexpected character {other:?}"),
                    ))
                }
            };
            self.note_token_at(pos.line);
            out.push(Spanned { tok, pos });
        }
    }

    /// Lex an `@`: the atomic `@std-sys` nodule-header marker (M-661) if `@` is immediately followed
    /// by the literal `std-sys`, else the bare guarantee-annotation [`Tok::At`]. The match is on the
    /// exact 7-char tail `std-sys`; a longer identifier (`std-system`) is **not** matched (the char
    /// after `-sys` must not be an identifier continuation), so the special case stays maximally
    /// narrow and `@std` / `@Exact` are unaffected. No `unsafe`; a pure lookahead-and-consume.
    fn lex_at(&mut self) -> Tok {
        // Consume '@', then peek the exact `std-sys` tail without consuming unless it matches in full.
        self.bump();
        const MARKER: &[char] = &['s', 't', 'd', '-', 's', 'y', 's'];
        let matches_tail = MARKER
            .iter()
            .enumerate()
            .all(|(k, &want)| self.chars.get(self.i + k).copied() == Some(want));
        // It must be a *whole* word: the char after `std-sys` cannot continue an identifier
        // (so `@std-system` is NOT the marker — it stays `@` + ident, which then fails downstream).
        let next_after = self.chars.get(self.i + MARKER.len()).copied();
        let whole_word = next_after.is_none_or(|c| !is_ident_continue(c));
        if matches_tail && whole_word {
            for _ in 0..MARKER.len() {
                self.bump();
            }
            Tok::AtStdSys
        } else {
            Tok::At
        }
    }

    fn single(&mut self, tok: Tok) -> Tok {
        self.bump();
        tok
    }

    fn lex_eq(&mut self) -> Tok {
        self.bump(); // '='
        if self.peek() == Some('>') {
            self.bump();
            Tok::FatArrow
        } else {
            Tok::Eq
        }
    }

    fn lex_dash(&mut self, pos: Pos) -> Result<Tok, ParseError> {
        self.bump(); // '-'
        if self.peek() == Some('>') {
            self.bump();
            Ok(Tok::Arrow)
        } else {
            // A bare '-' is not part of v0's surface outside trit literals/arrows.
            Err(ParseError::new(
                pos,
                "unexpected '-' (expected '->')".to_owned(),
            ))
        }
    }

    fn lex_angle_or_trit(&mut self, pos: Pos) -> Result<Tok, ParseError> {
        // One-char lookahead disambiguates trit-literal from type-args open.
        if matches!(self.peek2(), Some('+' | '-' | '0')) {
            self.bump(); // '<'
            let mut trits = String::new();
            loop {
                match self.peek() {
                    Some('>') => {
                        self.bump();
                        return Ok(Tok::TritLit(trits));
                    }
                    Some(t @ ('+' | '-' | '0')) => {
                        trits.push(t);
                        self.bump();
                    }
                    Some(bad) => {
                        return Err(ParseError::new(
                            self.pos(),
                            format!("balanced-ternary literal has non-trit glyph {bad:?}"),
                        ))
                    }
                    None => {
                        return Err(ParseError::new(
                            pos,
                            "unterminated balanced-ternary literal (missing '>')".to_owned(),
                        ))
                    }
                }
            }
        }
        Ok(self.single(Tok::LAngle))
    }

    fn lex_binary(&mut self) -> Tok {
        self.bump(); // '0'
        self.bump(); // 'b'
        let mut digits = String::new();
        while let Some(c) = self.peek() {
            if c == '0' || c == '1' || c == '_' {
                digits.push(c);
                self.bump();
            } else {
                break;
            }
        }
        Tok::BinLit(digits)
    }

    fn lex_int(&mut self, pos: Pos) -> Result<Tok, ParseError> {
        let mut s = String::new();
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                s.push(c);
                self.bump();
            } else {
                break;
            }
        }
        s.parse::<i64>()
            .map(Tok::Int)
            .map_err(|_| ParseError::new(pos, format!("integer literal out of range: {s}")))
    }

    fn lex_ident(&mut self) -> Tok {
        let mut s = String::new();
        while let Some(c) = self.peek() {
            if is_ident_continue(c) {
                s.push(c);
                self.bump();
            } else {
                break;
            }
        }
        keyword(&s).unwrap_or(Tok::Ident(s))
    }

    /// Skip whitespace and `//` line comments, capturing each comment into `self.comments`.
    ///
    /// Whitespace skipping is unchanged.  For every `//`-to-EOL run, the text (from `//` through
    /// the last non-newline character) is stored as a [`Comment`] in source order.  No comment
    /// is silently dropped (G2).
    fn skip_trivia(&mut self) {
        loop {
            match self.peek() {
                Some(c) if c.is_whitespace() => {
                    self.bump();
                }
                Some('/') if self.peek2() == Some('/') => {
                    // Record position of the `//` opener before consuming anything.
                    let comment_line = self.line;
                    let comment_col = self.col;
                    let trailing = self.last_token_line == comment_line;
                    // Consume the comment text up to (but not including) the newline.
                    let mut text = String::new();
                    while let Some(c) = self.peek() {
                        // Stop at the line terminator — break on `\r` too so a CRLF source does not
                        // leave a trailing `\r` in the comment text (the `\r\n` is then consumed by
                        // the whitespace arm). Keeps comment text `\r`-free + LF/CRLF round-trip
                        // parity, per the lexer's "no carriage-return" contract (Copilot #397).
                        if c == '\n' || c == '\r' {
                            break;
                        }
                        text.push(c);
                        self.bump();
                    }
                    self.comments.push(Comment {
                        text,
                        line: comment_line,
                        col: comment_col,
                        trailing,
                    });
                }
                _ => return,
            }
        }
    }
}

fn is_ident_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}

fn is_ident_continue(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

#[cfg(test)]
mod tests {
    //! Lexer-level tests covering:
    //!
    //! 1. The M-661 `@std-sys` atomic marker (pre-existing).
    //! 2. Comment capture via [`lex_with_comments`] (added for mycfmt comment-preservation,
    //!    Stage 1): cases (a) full-line, (b) trailing same-line, (c) multiple consecutive leading,
    //!    (d) comment separated by a blank line, (e) structured header lines (`// nodule:`/`// @key:`).
    //! 3. Token-stream equivalence: `lex(s) == lex_with_comments(s).0` for every input used here.
    use super::*;

    fn toks(src: &str) -> Vec<Tok> {
        lex(src)
            .expect("lexes")
            .into_iter()
            .map(|s| s.tok)
            .collect()
    }

    /// Assert that `lex` and `lex_with_comments` produce identical token streams (byte-for-byte).
    fn assert_token_stream_equiv(src: &str) {
        let plain = lex(src).expect("lex succeeded");
        let (with_cmt, _) = lex_with_comments(src).expect("lex_with_comments succeeded");
        assert_eq!(
            plain, with_cmt,
            "token stream mismatch between lex and lex_with_comments for input {src:?}"
        );
    }

    /// Copilot #397: on a CRLF source the captured comment text must not retain the trailing `\r`
    /// (the lexer's "no carriage-return" contract; LF/CRLF round-trip parity).
    #[test]
    fn comment_capture_strips_trailing_cr_on_crlf() {
        let (_toks, comments) =
            lex_with_comments("nodule d\r\nfn f() -> Binary{1} = 0b1 // why\r\n")
                .expect("lex_with_comments succeeded");
        let last = comments.last().expect("a comment was captured");
        assert_eq!(
            last.text, "// why",
            "comment text must be CR-free on CRLF input"
        );
        assert!(
            !last.text.contains('\r'),
            "no carriage-return in comment text"
        );
    }

    // -------------------------------------------------------------------------
    // Pre-existing @std-sys marker tests (unchanged)
    // -------------------------------------------------------------------------

    #[test]
    fn at_std_sys_lexes_as_one_atomic_marker_token() {
        // `@std-sys` is one token, immediately after the nodule path's last segment.
        let ts = toks("nodule std.sys.fs @std-sys");
        assert!(
            ts.contains(&Tok::AtStdSys),
            "expected an AtStdSys token, got: {ts:?}"
        );
        // And it is NOT split into `@` + ident (no bare `Tok::At` here).
        assert!(
            !ts.contains(&Tok::At),
            "must not also emit a bare `@`: {ts:?}"
        );
    }

    #[test]
    fn a_bare_at_is_still_the_guarantee_glyph() {
        // `@ Exact` (a guarantee annotation) stays `Tok::At` + the strength keyword — the special case
        // for `@std-sys` must not perturb the existing `T @ g` form.
        let ts = toks("Binary{8} @ Exact");
        assert!(ts.contains(&Tok::At), "expected a bare `@`, got: {ts:?}");
        assert!(
            !ts.contains(&Tok::AtStdSys),
            "a guarantee `@` must not be the std-sys marker: {ts:?}"
        );
    }

    #[test]
    fn at_std_sys_is_a_whole_word_only() {
        // `@std-system` is NOT the marker: it is `@` + ident `std` + `-` … (the `-` then fails the
        // lexer as it is not `->`), so the special case is maximally narrow. We assert the lexer does
        // not produce `AtStdSys` for it (it errors on the `-`, which proves it never matched the
        // marker — never a silent over-match, G2).
        assert!(
            lex("@std-system").is_err(),
            "`@std-system` must not lex as the `@std-sys` marker (whole-word only)"
        );
        // `@std` (no `-sys` tail) is a bare `@` + the identifier `std`.
        let ts = toks("@std");
        assert_eq!(ts, vec![Tok::At, Tok::Ident("std".to_owned()), Tok::Eof]);
    }

    // -------------------------------------------------------------------------
    // Token-stream equivalence (regression guard)
    // -------------------------------------------------------------------------

    #[test]
    fn lex_with_comments_token_stream_equals_lex() {
        // For every input exercised in this module, the token stream from lex_with_comments must
        // be byte-identical to the token stream from lex (comments do not bleed into tokens).
        let inputs = [
            "nodule std.sys.fs @std-sys",
            "Binary{8} @ Exact",
            "@std",
            // comment cases used below
            "// full-line comment\nnodule demo",
            "nodule demo  // trailing",
            "// first\n// second\nnodule demo",
            "// before\n\n// after blank\nnodule demo",
            "// nodule: foo\n// @matured: true\nnodule demo",
        ];
        for src in inputs {
            assert_token_stream_equiv(src);
        }
    }

    // -------------------------------------------------------------------------
    // Case (a): a full-line comment
    // -------------------------------------------------------------------------

    #[test]
    fn comment_capture_case_a_full_line_comment() {
        // A `//` comment occupying the whole line is captured with trailing=false.
        let src = "// full-line comment\nnodule demo";
        let (toks, comments) = lex_with_comments(src).expect("lex_with_comments succeeded");
        assert_eq!(comments.len(), 1, "exactly one comment: {comments:?}");
        let c = &comments[0];
        assert_eq!(c.text, "// full-line comment");
        assert_eq!(c.line, 1, "comment is on line 1");
        assert_eq!(c.col, 1, "comment starts at col 1");
        assert!(!c.trailing, "full-line comment must have trailing=false");
        // Tokens: nodule, ident("demo"), eof — no comment token.
        let tok_kinds: Vec<_> = toks.iter().map(|s| &s.tok).collect();
        assert!(
            tok_kinds.contains(&&Tok::Nodule),
            "nodule token must be present"
        );
        assert!(
            !tok_kinds
                .iter()
                .any(|t| matches!(t, Tok::Ident(s) if s == "// full-line comment")),
            "comment must not appear in token stream"
        );
    }

    // -------------------------------------------------------------------------
    // Case (b): a trailing same-line comment
    // -------------------------------------------------------------------------

    #[test]
    fn comment_capture_case_b_trailing_same_line_comment() {
        // A comment after a real token on the same line: trailing=true.
        let src = "nodule demo  // trailing";
        let (_toks, comments) = lex_with_comments(src).expect("lex_with_comments succeeded");
        assert_eq!(comments.len(), 1, "exactly one comment: {comments:?}");
        let c = &comments[0];
        assert_eq!(c.text, "// trailing");
        assert_eq!(c.line, 1, "comment is on line 1");
        assert!(
            c.trailing,
            "comment after tokens on same line must have trailing=true"
        );
    }

    // -------------------------------------------------------------------------
    // Case (c): multiple consecutive leading comments
    // -------------------------------------------------------------------------

    #[test]
    fn comment_capture_case_c_consecutive_leading_comments() {
        // Two `//` lines in a row: both captured, both trailing=false, in source order.
        let src = "// first\n// second\nnodule demo";
        let (_toks, comments) = lex_with_comments(src).expect("lex_with_comments succeeded");
        assert_eq!(comments.len(), 2, "two comments expected: {comments:?}");
        assert_eq!(comments[0].text, "// first");
        assert_eq!(comments[0].line, 1);
        assert!(!comments[0].trailing);
        assert_eq!(comments[1].text, "// second");
        assert_eq!(comments[1].line, 2);
        assert!(!comments[1].trailing);
    }

    // -------------------------------------------------------------------------
    // Case (d): comment separated by a blank line
    // -------------------------------------------------------------------------

    #[test]
    fn comment_capture_case_d_comment_after_blank_line() {
        // A comment on line 1, a blank line 2, a comment on line 3, then code.
        // Both comments are captured; neither is trailing.
        let src = "// before\n\n// after blank\nnodule demo";
        let (_toks, comments) = lex_with_comments(src).expect("lex_with_comments succeeded");
        assert_eq!(comments.len(), 2, "two comments expected: {comments:?}");
        assert_eq!(comments[0].text, "// before");
        assert_eq!(comments[0].line, 1);
        assert!(!comments[0].trailing);
        assert_eq!(comments[1].text, "// after blank");
        assert_eq!(
            comments[1].line, 3,
            "comment after blank line must be on line 3"
        );
        assert!(!comments[1].trailing);
    }

    // -------------------------------------------------------------------------
    // Case (e): structured header lines (// nodule:, // @key:) — captured verbatim
    // -------------------------------------------------------------------------

    #[test]
    fn comment_capture_case_e_structured_header_comments() {
        // Header-style comments (`// nodule:`, `// @matured: true`) are captured verbatim.
        // Their placement (before/after the `nodule` declaration) is for the formatter to decide;
        // the lexer just captures them all without interpretation.
        let src = "// nodule: foo\n// @matured: true\nnodule demo";
        let (_toks, comments) = lex_with_comments(src).expect("lex_with_comments succeeded");
        assert_eq!(comments.len(), 2, "two header comments: {comments:?}");
        assert_eq!(comments[0].text, "// nodule: foo");
        assert_eq!(comments[0].line, 1);
        assert!(!comments[0].trailing);
        assert_eq!(comments[1].text, "// @matured: true");
        assert_eq!(comments[1].line, 2);
        assert!(!comments[1].trailing);
    }

    // -------------------------------------------------------------------------
    // Additional edge-cases: end-of-file comment, empty comment, no comments
    // -------------------------------------------------------------------------

    #[test]
    fn comment_capture_eof_without_trailing_newline() {
        // A comment at end-of-file (no trailing newline) is still captured.
        let src = "nodule demo\n// last line no newline";
        let (_toks, comments) = lex_with_comments(src).expect("lex_with_comments succeeded");
        assert_eq!(comments.len(), 1, "one comment: {comments:?}");
        assert_eq!(comments[0].text, "// last line no newline");
        assert!(!comments[0].trailing, "no tokens on that line => leading");
    }

    #[test]
    fn comment_capture_empty_comment_body() {
        // `//` with nothing after it (just a newline) stores exactly `"//"`.
        let src = "//\nnodule demo";
        let (_toks, comments) = lex_with_comments(src).expect("lex_with_comments succeeded");
        assert_eq!(comments.len(), 1);
        assert_eq!(
            comments[0].text, "//",
            "empty comment body: text must be exactly \"//\""
        );
        assert!(!comments[0].trailing);
    }

    #[test]
    fn comment_capture_no_comments_gives_empty_vec() {
        // Source with no comments yields an empty comment vec.
        let src = "nodule demo\nfn f() -> Binary{8} = 0b0";
        let (_toks, comments) = lex_with_comments(src).expect("lex_with_comments succeeded");
        assert!(
            comments.is_empty(),
            "no comments in source => empty vec: {comments:?}"
        );
    }
}
