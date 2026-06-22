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
                '@' => self.single(Tok::At),
                ':' => self.single(Tok::Colon),
                ',' => self.single(Tok::Comma),
                '.' => self.single(Tok::Dot),
                '|' => self.single(Tok::Pipe),
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
