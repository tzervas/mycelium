//! The L1 lexer (RFC-0006; DN-02). Hand-written, no dependencies (house style). Produces a token
//! stream or an explicit [`ParseError`] — never a silent skip of an unrecognized character.
//!
//! The one subtlety is `<`: it opens both a balanced-ternary literal (`<+0->`) and a type-argument
//! list (`List<Ternary{6}>`). They are disambiguated by one character of lookahead — a `<`
//! immediately followed by a trit glyph (`+`, `-`, `0`) is a ternary literal (scanned whole),
//! anything else is the [`Tok::LAngle`] punctuation. A literal with a non-trit glyph before its
//! closing `>` is an explicit error (`reject/04`), never a silent truncation.

use crate::error::ParseError;
use crate::token::{keyword, Pos, Spanned, Tok};

struct Lexer {
    chars: Vec<char>,
    i: usize,
    line: u32,
    col: u32,
}

/// Tokenize `src` into a [`Spanned`] stream terminated by [`Tok::Eof`].
pub fn lex(src: &str) -> Result<Vec<Spanned>, ParseError> {
    let mut lx = Lexer {
        chars: src.chars().collect(),
        i: 0,
        line: 1,
        col: 1,
    };
    lx.run()
}

impl Lexer {
    fn pos(&self) -> Pos {
        Pos {
            line: self.line,
            col: self.col,
        }
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

    /// Skip whitespace and `//` line comments.
    fn skip_trivia(&mut self) {
        loop {
            match self.peek() {
                Some(c) if c.is_whitespace() => {
                    self.bump();
                }
                Some('/') if self.peek2() == Some('/') => {
                    while let Some(c) = self.peek() {
                        if c == '\n' {
                            break;
                        }
                        self.bump();
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
    //! Lexer-level tests for the M-661 `@std-sys` atomic marker. `@std-sys` cannot lex as `@` + an
    //! identifier (the `-` is not an identifier char), so it is recognized whole as [`Tok::AtStdSys`];
    //! the special case must stay maximally narrow (the bare `@` guarantee glyph and `@std` / a longer
    //! `@std-system` are unaffected). These pin that boundary directly (the parser/checker tests cover
    //! the surface behavior; this covers the tokenizer in isolation — defense in depth for an FFI gate).
    use super::*;

    fn toks(src: &str) -> Vec<Tok> {
        lex(src)
            .expect("lexes")
            .into_iter()
            .map(|s| s.tok)
            .collect()
    }

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
}
