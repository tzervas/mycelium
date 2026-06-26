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
    /// Comments collected so far, in source order.  Populated during `skip_trivia` calls
    /// **only when [`capture_comments`](Self::capture_comments) is set** (the `mycfmt` path).
    comments: Vec<Comment>,
    /// The source line on which the most-recently-emitted non-comment token started, or `0`
    /// if no token has been emitted yet.  Used to compute [`Comment::trailing`].
    last_token_line: u32,
    /// When `false` (the plain [`lex`] path), `//` comments are skipped without allocating a
    /// [`Comment`] — the common parse/check front-end pays no comment-capture cost. When `true`
    /// (the [`lex_with_comments`] / `mycfmt` path), each comment is captured into `comments`.
    capture_comments: bool,
}

/// Tokenize `src` into a [`Spanned`] stream terminated by [`Tok::Eof`].
///
/// Comments are **discarded** (behavior-identical to the original implementation).  This is the
/// front-end fast path: `//` comments are *skipped without allocating* a [`Comment`], so a
/// parse/check that never needs comments pays no capture cost (Copilot #397).  Use
/// [`lex_with_comments`] to obtain the comment side-table alongside the same token stream.
pub fn lex(src: &str) -> Result<Vec<Spanned>, ParseError> {
    run_lexer(src, false).map(|(toks, _comments)| toks)
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
    run_lexer(src, true)
}

/// Shared lexer driver. `capture_comments` selects the fast path ([`lex`], comments skipped without
/// allocation) vs the `mycfmt` path ([`lex_with_comments`], comments captured into the side-table).
/// The token stream is identical either way (comments never enter the token stream).
fn run_lexer(
    src: &str,
    capture_comments: bool,
) -> Result<(Vec<Spanned>, Vec<Comment>), ParseError> {
    let mut lx = Lexer {
        chars: src.chars().collect(),
        i: 0,
        line: 1,
        col: 1,
        comments: Vec::new(),
        last_token_line: 0,
        capture_comments,
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
                // `|` is the sum-type constructor separator (`type T = A | B`) and the bitwise-`bor`
                // operator (RFC-0025 / M-705); `||` is the logical-`or` operator. The parser
                // disambiguates single `|` by position (type-decl separator vs expression operator).
                // (There is no `|`-separated pattern-alternation production in the v0 surface.)
                '|' => self.lex_pipe(),
                // `!` opens the effect annotation `!{ … }` (RFC-0014 §3.4; M-660) and is the unary
                // `not` operator at expression position (RFC-0025 / M-705); `!=` is the `ne`
                // operator. The parser accepts a signature `!` only before `{` (never a silent
                // accept, G2).
                '!' => self.lex_bang(),
                // `+` is the trait-bound separator (`T: A + B`; RFC-0019 §4.1) and the infix `add`
                // operator at expression position (RFC-0025 / M-705). It is also a trit glyph, but a
                // trit literal is only ever scanned *whole* from an opening `<` (in
                // `lex_angle_or_trit`), so a `+` reaching here is one of those two operator tokens.
                '+' => self.single(Tok::Plus),
                // `*` is the glob marker of a wildcard import `use a.b.*` (M-662) and the infix
                // `mul` operator at expression position (RFC-0025 / M-705); the parser disambiguates
                // by position.
                '*' => self.single(Tok::Star),
                // `/` is the infix `div` operator (RFC-0025 / M-705). `//` line comments are
                // consumed by `skip_trivia`, so a `/` reaching here is always the operator.
                '/' => self.single(Tok::Slash),
                // `%` (rem), `^` (xor): infix operators at expression position (RFC-0025 / M-705).
                '%' => self.single(Tok::Percent),
                '^' => self.single(Tok::Caret),
                // `&` (band) / `&&` (and): bitwise- and logical-and operators (RFC-0025 / M-705).
                '&' => self.lex_amp(),
                '<' => self.lex_angle_or_trit(pos)?,
                '=' => self.lex_eq(),
                '-' => self.lex_dash(),
                '0' if self.peek2() == Some('b') => self.lex_binary(pos)?,
                // `0x…` is a byte-string literal (RFC-0032 D4, M-750), lexed whole like `0b…`. The
                // `0x` prefix is unambiguous: a bare `0` not followed by `b`/`x` is an int.
                '0' if self.peek2() == Some('x') => self.lex_hex_bytes(pos)?,
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
        match self.peek() {
            Some('>') => {
                self.bump();
                Tok::FatArrow
            }
            // `==` is the infix `eq` operator (RFC-0025 / M-705); `=` stays the binder glyph.
            Some('=') => {
                self.bump();
                Tok::EqEq
            }
            _ => Tok::Eq,
        }
    }

    fn lex_dash(&mut self) -> Tok {
        self.bump(); // '-'
        if self.peek() == Some('>') {
            self.bump();
            // `->` is the function-type arrow (RFC-0024 §3).
            Tok::Arrow
        } else {
            // A bare `-` is the infix sub / unary neg operator (RFC-0025 / M-705); the parser
            // disambiguates binary from prefix by position.
            Tok::Minus
        }
    }

    /// `&` (band) or `&&` (logical and) — RFC-0025 / M-705.
    fn lex_amp(&mut self) -> Tok {
        self.bump(); // '&'
        if self.peek() == Some('&') {
            self.bump();
            Tok::AmpAmp
        } else {
            Tok::Amp
        }
    }

    /// `|` (sum-type constructor separator / `bor`) or `||` (logical or) — RFC-0025 / M-705.
    fn lex_pipe(&mut self) -> Tok {
        self.bump(); // '|'
        if self.peek() == Some('|') {
            self.bump();
            Tok::PipePipe
        } else {
            Tok::Pipe
        }
    }

    /// `!` (effect-set opener / unary not) or `!=` (ne) — RFC-0014 §3.4 / RFC-0025 / M-705.
    fn lex_bang(&mut self) -> Tok {
        self.bump(); // '!'
        if self.peek() == Some('=') {
            self.bump();
            Tok::BangEq
        } else {
            Tok::Bang
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

    fn lex_binary(&mut self, pos: Pos) -> Result<Tok, ParseError> {
        self.bump(); // '0'
        self.bump(); // 'b'
        let mut digits = String::new();
        // Track whether any actual binary digit (not just a `_` separator) was scanned: a
        // base-prefixed literal must carry a value. `0b` alone, or `0b_`, is a never-silent
        // lex error (G2) — the literal is parsed only when it has at least one `0`/`1`.
        let mut saw_digit = false;
        while let Some(c) = self.peek() {
            if c == '0' || c == '1' {
                saw_digit = true;
                digits.push(c);
                self.bump();
            } else if c == '_' {
                digits.push(c);
                self.bump();
            } else {
                break;
            }
        }
        if !saw_digit {
            return Err(ParseError::new(
                pos,
                "binary literal `0b` has no digits (expected at least one `0` or `1`)".to_owned(),
            ));
        }
        Ok(Tok::BinLit(digits))
    }

    /// Lex a byte-string literal `0x…` (RFC-0032 D4, M-750), mirroring [`Self::lex_binary`]: scan
    /// hex digits + `_` separators verbatim into the inner string. Never-silent (G2): an empty `0x`
    /// (no hex digit), a non-hex digit, **or an odd number of hex digits** (a byte is two hex chars)
    /// is an explicit [`ParseError`] naming the offending position — never a silently-empty or
    /// half-byte token. The `_` separators are preserved but do not count toward the byte parity.
    fn lex_hex_bytes(&mut self, pos: Pos) -> Result<Tok, ParseError> {
        self.bump(); // '0'
        self.bump(); // 'x'
        let mut digits = String::new();
        // Count actual hex digits (not `_` separators) for the even-parity check; a base-prefixed
        // literal must carry at least one digit (`0x` / `0x_` alone is a never-silent lex error).
        let mut hex_count = 0usize;
        while let Some(c) = self.peek() {
            if c.is_ascii_hexdigit() {
                hex_count += 1;
                digits.push(c);
                self.bump();
            } else if c == '_' {
                digits.push(c);
                self.bump();
            } else {
                break;
            }
        }
        if hex_count == 0 {
            return Err(ParseError::new(
                pos,
                "byte-string literal `0x` has no hex digits (expected at least one, even count)"
                    .to_owned(),
            ));
        }
        if hex_count % 2 != 0 {
            return Err(ParseError::new(
                pos,
                format!(
                    "byte-string literal `0x` has an odd hex-digit count ({hex_count}) — each byte \
                     is two hex chars (RFC-0032 D4); never a silent half-byte"
                ),
            ));
        }
        Ok(Tok::BytesLit(digits))
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
                    // Build the comment text only on the capture path; on the plain `lex` path
                    // `text` stays `None`, so we advance past the comment with no `String` alloc
                    // and no `Comment` push (Copilot #397 — the front-end pays no comment cost).
                    let mut text = self.capture_comments.then(String::new);
                    while let Some(c) = self.peek() {
                        // Stop at the line terminator — break on `\r` too so a CRLF source does not
                        // leave a trailing `\r` in the comment text (the `\r\n` is then consumed by
                        // the whitespace arm). Keeps comment text `\r`-free + LF/CRLF round-trip
                        // parity, per the lexer's "no carriage-return" contract (Copilot #397).
                        if c == '\n' || c == '\r' {
                            break;
                        }
                        if let Some(t) = text.as_mut() {
                            t.push(c);
                        }
                        self.bump();
                    }
                    if let Some(text) = text {
                        self.comments.push(Comment {
                            text,
                            line: comment_line,
                            col: comment_col,
                            trailing,
                        });
                    }
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

    /// Copilot #397 (perf): `lex` is the front-end fast path — it skips `//` comments without
    /// building a [`Comment`], while [`lex_with_comments`] still captures them. The capture flag
    /// never changes the token stream. This pins that the fast path coexists with capture rather
    /// than routing every parse/check through comment allocation.
    #[test]
    fn lex_fast_path_skips_comments_capture_path_keeps_them() {
        let src = "// doc: why\nnodule demo  // trailing why\nfn f() -> Binary{1} = 0b1 // more";
        let fast = lex(src).expect("lex (fast path) succeeded");
        let (cap, comments) = lex_with_comments(src).expect("lex_with_comments succeeded");
        assert_eq!(
            fast, cap,
            "the capture flag must not change the token stream"
        );
        assert_eq!(
            comments.len(),
            3,
            "all three // comments captured: {comments:?}"
        );
        assert!(
            !fast
                .iter()
                .any(|s| matches!(&s.tok, Tok::Ident(t) if t.starts_with("//"))),
            "no comment text leaks into the fast-path token stream"
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
        // `@std-system` is NOT the marker: it lexes as `@` + ident `std` + `-` (the infix `sub`
        // operator, RFC-0025 / M-705) + ident `system`, so the special case is maximally narrow.
        // We assert the lexer never produces `AtStdSys` for it (never a silent over-match, G2).
        let ts = toks("@std-system");
        assert!(
            !ts.contains(&Tok::AtStdSys),
            "`@std-system` must not lex as the `@std-sys` marker (whole-word only): {ts:?}"
        );
        assert_eq!(
            ts,
            vec![
                Tok::At,
                Tok::Ident("std".to_owned()),
                Tok::Minus,
                Tok::Ident("system".to_owned()),
                Tok::Eof,
            ],
            "`@std-system` lexes as `@ std - system`"
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

    // -------------------------------------------------------------------------
    // DN-40 §3 (low) — base-prefixed literal with no digits is a never-silent
    // lex error (G2). A `0b` prefix that scans no `0`/`1` is rejected, naming the
    // offending position; valid binary / trit literals are unaffected.
    // -------------------------------------------------------------------------

    /// `0b` with no following binary digit is a lex error, not a silently-empty `BinLit`.
    #[test]
    fn lex_binary_empty_literal_is_a_lex_error() {
        let err = lex("0b").expect_err("`0b` alone must be a lex error");
        assert!(
            err.to_string().contains("no digits"),
            "error must name the empty-binary-literal cause: {err}"
        );
    }

    /// `0b` followed only by a `_` separator (no actual `0`/`1`) carries no value and is rejected.
    #[test]
    fn lex_binary_only_separator_is_a_lex_error() {
        lex("0b_").expect_err("`0b_` (separator, no digit) must be a lex error");
    }

    /// `0b` at end of a larger source still errors (no digit before EOF), and the error is raised
    /// rather than emitting an empty token (never-silent).
    #[test]
    fn lex_binary_empty_literal_in_context_is_a_lex_error() {
        lex("fn f() -> Binary{1} = 0b")
            .expect_err("trailing `0b` with no digit must be a lex error");
    }

    /// Valid binary literals still lex to a `BinLit` carrying their digits (regression guard).
    #[test]
    fn lex_binary_valid_literals_still_lex() {
        assert_eq!(toks("0b0"), vec![Tok::BinLit("0".to_owned()), Tok::Eof]);
        assert_eq!(toks("0b1"), vec![Tok::BinLit("1".to_owned()), Tok::Eof]);
        assert_eq!(
            toks("0b1010"),
            vec![Tok::BinLit("1010".to_owned()), Tok::Eof]
        );
        // A `_` separator is allowed once at least one real digit is present.
        assert_eq!(toks("0b1_0"), vec![Tok::BinLit("1_0".to_owned()), Tok::Eof]);
    }

    /// RFC-0032 D4 (M-750): `0x…` byte-string literals lex like `0b…`. A valid even-hex literal
    /// (optionally with `_` separators) lexes to a `BytesLit` carrying its inner string verbatim.
    #[test]
    fn lex_hex_bytes_valid_literals() {
        assert_eq!(toks("0x48"), vec![Tok::BytesLit("48".to_owned()), Tok::Eof]);
        assert_eq!(
            toks("0x48_65_6c_6c_6f"),
            vec![Tok::BytesLit("48_65_6c_6c_6f".to_owned()), Tok::Eof]
        );
        // Uppercase hex digits are accepted too.
        assert_eq!(
            toks("0xDEAD_BEEF"),
            vec![Tok::BytesLit("DEAD_BEEF".to_owned()), Tok::Eof]
        );
    }

    /// Never-silent (G2): an **odd** hex-digit count is a lex error — a byte is two hex chars, never
    /// a silent half-byte.
    #[test]
    fn lex_hex_bytes_odd_count_is_a_lex_error() {
        let err = lex("0x123").expect_err("`0x123` (odd hex count) must be a lex error");
        assert!(
            err.to_string().contains("odd hex-digit count"),
            "error must name the odd-hex cause: {err}"
        );
        // A `_` separator does not change parity: `0x1_2_3` is still three hex digits → error.
        lex("0x1_2_3").expect_err("`0x1_2_3` (three hex digits) must be a lex error");
    }

    /// Never-silent (G2): an empty `0x` (no hex digit, or only a separator) is a lex error.
    #[test]
    fn lex_hex_bytes_empty_is_a_lex_error() {
        let err = lex("0x").expect_err("`0x` alone must be a lex error");
        assert!(
            err.to_string().contains("no hex digits"),
            "error must name the empty cause: {err}"
        );
        lex("0x_").expect_err("`0x_` (separator, no digit) must be a lex error");
    }

    /// A non-hex digit terminates the hex scan; `0x1g` lexes the even-hex `0x1`?-no: `1` alone is
    /// odd → error. `0x12g` lexes `0x12` (even) then the identifier `g`. This pins that the hex scan
    /// stops at a non-hex char rather than consuming it (and the parity check applies to what it took).
    #[test]
    fn lex_hex_bytes_stops_at_non_hex() {
        // `0x12` is even → BytesLit("12"), then `g` is an identifier.
        assert_eq!(
            toks("0x12g"),
            vec![
                Tok::BytesLit("12".to_owned()),
                Tok::Ident("g".to_owned()),
                Tok::Eof
            ]
        );
        // `0x1g` took only `1` (odd) before the non-hex `g` → a lex error (never a silent half-byte).
        lex("0x1g").expect_err("`0x1g` (one hex digit then non-hex) must be a lex error");
    }

    /// `Seq` and `Bytes` lex as keywords (never silent identifiers — G2).
    #[test]
    fn lex_seq_and_bytes_keywords() {
        assert_eq!(toks("Seq"), vec![Tok::Seq, Tok::Eof]);
        assert_eq!(toks("Bytes"), vec![Tok::Bytes, Tok::Eof]);
    }

    /// Trit literals use the `<…>` angle form (not a `0t` prefix); the empty case `<>` is already
    /// safe (it falls through to `LAngle`, never an empty `TritLit`). Valid trit literals still lex.
    /// (See FLAG in the leaf report: this lexer has no `0t` literal; `0t` lexes as int `0` + ident.)
    #[test]
    fn lex_trit_valid_literals_still_lex() {
        assert_eq!(
            toks("<+-0>"),
            vec![Tok::TritLit("+-0".to_owned()), Tok::Eof]
        );
        assert_eq!(toks("<0>"), vec![Tok::TritLit("0".to_owned()), Tok::Eof]);
        // `<>` is NOT a trit literal: lookahead fails, so it lexes as the `<`/`>` angle pair.
        assert_eq!(toks("<>"), vec![Tok::LAngle, Tok::RAngle, Tok::Eof]);
    }
}
