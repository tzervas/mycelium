//! The L1 recursive-descent parser (RFC-0006; faithful to `docs/spec/grammar/mycelium.ebnf`).
//! Hand-written, no dependencies. Every failure is an explicit [`ParseError`] with a position
//! (never a panic, never a silent accept — S5/G2). v0 covers the L1-facing core.

use crate::ast::{
    Arm, BaseType, Colony, Ctor, Expr, FnDecl, FnSig, Item, Literal, Param, Path, Pattern, Scalar,
    Sparsity, Strength, TraitDecl, TypeDecl, TypeRef,
};
use crate::error::ParseError;
use crate::lexer::lex;
use crate::token::{Pos, ScalarTok, Spanned, StrengthTok, Tok};

/// Parse a complete `colony` program from source.
pub fn parse(src: &str) -> Result<Colony, ParseError> {
    let toks = lex(src)?;
    let mut p = Parser { toks, i: 0 };
    let colony = p.parse_colony()?;
    p.expect(&Tok::Eof, "end of input")?;
    Ok(colony)
}

struct Parser {
    toks: Vec<Spanned>,
    i: usize,
}

impl Parser {
    fn cur(&self) -> &Tok {
        &self.toks[self.i].tok
    }

    fn pos(&self) -> Pos {
        self.toks[self.i].pos
    }

    fn at(&self, t: &Tok) -> bool {
        self.cur() == t
    }

    fn bump(&mut self) -> Tok {
        let t = self.toks[self.i].tok.clone();
        if self.i + 1 < self.toks.len() {
            self.i += 1;
        }
        t
    }

    fn err<T>(&self, what: &str) -> Result<T, ParseError> {
        Err(ParseError::new(
            self.pos(),
            format!("expected {what}, found {:?}", self.cur()),
        ))
    }

    fn expect(&mut self, t: &Tok, what: &str) -> Result<(), ParseError> {
        if self.at(t) {
            self.bump();
            Ok(())
        } else {
            self.err(what)
        }
    }

    fn eat(&mut self, t: &Tok) -> bool {
        if self.at(t) {
            self.bump();
            true
        } else {
            false
        }
    }

    fn ident(&mut self) -> Result<String, ParseError> {
        match self.cur() {
            Tok::Ident(s) => {
                let s = s.clone();
                self.bump();
                Ok(s)
            }
            _ => self.err("an identifier"),
        }
    }

    fn u32_lit(&mut self) -> Result<u32, ParseError> {
        match *self.cur() {
            Tok::Int(n) => {
                self.bump();
                u32::try_from(n)
                    .map_err(|_| ParseError::new(self.pos(), format!("{n} is out of u32 range")))
            }
            _ => self.err("a non-negative integer"),
        }
    }

    // ---- items ----

    fn parse_colony(&mut self) -> Result<Colony, ParseError> {
        self.expect(&Tok::Colony, "a `colony` header to open the program")?;
        let path = self.parse_path()?;
        let mut items = Vec::new();
        while !self.at(&Tok::Eof) {
            items.push(self.parse_item()?);
        }
        Ok(Colony { path, items })
    }

    fn parse_item(&mut self) -> Result<Item, ParseError> {
        match self.cur() {
            Tok::Use => {
                self.bump();
                Ok(Item::Use(self.parse_path()?))
            }
            Tok::Type => self.parse_type_decl().map(Item::Type),
            Tok::Trait => self.parse_trait_decl().map(Item::Trait),
            Tok::Matured | Tok::Fn => self.parse_fn_decl().map(Item::Fn),
            _ => self.err("a top-level item (`use`, `type`, `trait`, `fn`, or `matured fn`)"),
        }
    }

    fn parse_type_decl(&mut self) -> Result<TypeDecl, ParseError> {
        self.expect(&Tok::Type, "`type`")?;
        let name = self.ident()?;
        let params = self.parse_type_params_opt()?;
        self.expect(&Tok::Eq, "`=` before the constructors")?;
        let mut ctors = vec![self.parse_ctor()?];
        while self.eat(&Tok::Pipe) {
            ctors.push(self.parse_ctor()?);
        }
        Ok(TypeDecl {
            name,
            params,
            ctors,
        })
    }

    fn parse_ctor(&mut self) -> Result<Ctor, ParseError> {
        let name = self.ident()?;
        let mut fields = Vec::new();
        if self.eat(&Tok::LParen) {
            fields.push(self.parse_type_ref()?);
            while self.eat(&Tok::Comma) {
                fields.push(self.parse_type_ref()?);
            }
            self.expect(&Tok::RParen, "`)` to close the constructor fields")?;
        }
        Ok(Ctor { name, fields })
    }

    fn parse_trait_decl(&mut self) -> Result<TraitDecl, ParseError> {
        self.expect(&Tok::Trait, "`trait`")?;
        let name = self.ident()?;
        let params = self.parse_type_params_opt()?;
        self.expect(&Tok::LBrace, "`{` to open the trait body")?;
        let mut sigs = Vec::new();
        while !self.at(&Tok::RBrace) {
            sigs.push(self.parse_fn_sig()?);
        }
        self.expect(&Tok::RBrace, "`}` to close the trait body")?;
        Ok(TraitDecl { name, params, sigs })
    }

    fn parse_fn_sig(&mut self) -> Result<FnSig, ParseError> {
        self.expect(&Tok::Fn, "`fn` in the trait body")?;
        self.parse_sig_tail()
    }

    fn parse_fn_decl(&mut self) -> Result<FnDecl, ParseError> {
        let matured = self.eat(&Tok::Matured);
        self.expect(&Tok::Fn, "`fn`")?;
        let sig = self.parse_sig_tail()?;
        self.expect(&Tok::Eq, "`=` before the function body")?;
        let body = self.parse_expr()?;
        Ok(FnDecl { matured, sig, body })
    }

    /// The shared `name <params>? ( value_params? ) -> ret` tail of a signature.
    fn parse_sig_tail(&mut self) -> Result<FnSig, ParseError> {
        let name = self.ident()?;
        let params = self.parse_type_params_opt()?;
        self.expect(&Tok::LParen, "`(` to open the parameter list")?;
        let value_params = self.parse_params_opt()?;
        self.expect(&Tok::RParen, "`)` to close the parameter list")?;
        self.expect(&Tok::Arrow, "`->` and a result type")?;
        let ret = self.parse_type_ref()?;
        Ok(FnSig {
            name,
            params,
            value_params,
            ret,
        })
    }

    fn parse_params_opt(&mut self) -> Result<Vec<Param>, ParseError> {
        let mut params = Vec::new();
        if self.at(&Tok::RParen) {
            return Ok(params);
        }
        loop {
            let name = self.ident()?;
            self.expect(&Tok::Colon, "`:` and the parameter type")?;
            let ty = self.parse_type_ref()?;
            params.push(Param { name, ty });
            if !self.eat(&Tok::Comma) {
                break;
            }
        }
        Ok(params)
    }

    fn parse_type_params_opt(&mut self) -> Result<Vec<String>, ParseError> {
        let mut params = Vec::new();
        if self.eat(&Tok::LAngle) {
            params.push(self.ident()?);
            while self.eat(&Tok::Comma) {
                params.push(self.ident()?);
            }
            self.expect(&Tok::RAngle, "`>` to close the type parameters")?;
        }
        Ok(params)
    }

    // ---- types ----

    fn parse_type_ref(&mut self) -> Result<TypeRef, ParseError> {
        let base = self.parse_base_type()?;
        let guarantee = if self.eat(&Tok::At) {
            Some(self.parse_strength()?)
        } else {
            None
        };
        Ok(TypeRef { base, guarantee })
    }

    fn parse_base_type(&mut self) -> Result<BaseType, ParseError> {
        match self.cur().clone() {
            Tok::Binary => {
                self.bump();
                let w = self.braced_u32()?;
                Ok(BaseType::Binary(w))
            }
            Tok::Ternary => {
                self.bump();
                let t = self.braced_u32()?;
                Ok(BaseType::Ternary(t))
            }
            Tok::Dense => {
                self.bump();
                self.expect(&Tok::LBrace, "`{` after `Dense`")?;
                let dim = self.u32_lit()?;
                self.expect(&Tok::Comma, "`,` between dim and dtype")?;
                let scalar = self.parse_scalar()?;
                self.expect(&Tok::RBrace, "`}` to close `Dense{…}`")?;
                Ok(BaseType::Dense(dim, scalar))
            }
            Tok::Vsa => {
                self.bump();
                self.expect(&Tok::LBrace, "`{` after `VSA`")?;
                let model = self.ident()?;
                self.expect(&Tok::Comma, "`,` after the model")?;
                let dim = self.u32_lit()?;
                self.expect(&Tok::Comma, "`,` before the sparsity")?;
                let sparsity = self.parse_sparsity()?;
                self.expect(&Tok::RBrace, "`}` to close `VSA{…}`")?;
                Ok(BaseType::Vsa {
                    model,
                    dim,
                    sparsity,
                })
            }
            Tok::Substrate => {
                self.bump();
                self.expect(&Tok::LBrace, "`{` after `Substrate`")?;
                let name = self.ident()?;
                self.expect(&Tok::RBrace, "`}` to close `Substrate{…}`")?;
                Ok(BaseType::Substrate(name))
            }
            Tok::Ident(s) => {
                self.bump();
                let args = self.parse_type_args_opt()?;
                Ok(BaseType::Named(s, args))
            }
            _ => self.err("a type"),
        }
    }

    fn braced_u32(&mut self) -> Result<u32, ParseError> {
        self.expect(&Tok::LBrace, "`{` and a width")?;
        let n = self.u32_lit()?;
        self.expect(&Tok::RBrace, "`}` to close the width")?;
        Ok(n)
    }

    fn parse_type_args_opt(&mut self) -> Result<Vec<TypeRef>, ParseError> {
        let mut args = Vec::new();
        if self.eat(&Tok::LAngle) {
            args.push(self.parse_type_ref()?);
            while self.eat(&Tok::Comma) {
                args.push(self.parse_type_ref()?);
            }
            self.expect(&Tok::RAngle, "`>` to close the type arguments")?;
        }
        Ok(args)
    }

    fn parse_sparsity(&mut self) -> Result<Sparsity, ParseError> {
        match self.cur() {
            Tok::Dense => {
                self.bump();
                Ok(Sparsity::Dense)
            }
            Tok::Sparse => {
                self.bump();
                let k = self.braced_u32()?;
                Ok(Sparsity::Sparse(k))
            }
            _ => self.err("a sparsity (`Dense` or `Sparse{…}`)"),
        }
    }

    fn parse_scalar(&mut self) -> Result<Scalar, ParseError> {
        match *self.cur() {
            Tok::Scalar(s) => {
                self.bump();
                Ok(match s {
                    ScalarTok::F16 => Scalar::F16,
                    ScalarTok::Bf16 => Scalar::Bf16,
                    ScalarTok::F32 => Scalar::F32,
                    ScalarTok::F64 => Scalar::F64,
                })
            }
            _ => self.err("a scalar kind (`F16|BF16|F32|F64`)"),
        }
    }

    fn parse_strength(&mut self) -> Result<Strength, ParseError> {
        match *self.cur() {
            Tok::Strength(s) => {
                self.bump();
                Ok(match s {
                    StrengthTok::Exact => Strength::Exact,
                    StrengthTok::Proven => Strength::Proven,
                    StrengthTok::Empirical => Strength::Empirical,
                    StrengthTok::Declared => Strength::Declared,
                })
            }
            _ => self.err("a guarantee strength (`Exact|Proven|Empirical|Declared`)"),
        }
    }

    // ---- expressions ----

    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        self.teach_imperative()?;
        match self.cur() {
            Tok::Let => self.parse_let(),
            Tok::If => self.parse_if(),
            Tok::Match => self.parse_match(),
            Tok::For => self.parse_for(),
            Tok::Swap => self.parse_swap(),
            Tok::Wild => self.parse_wild(),
            Tok::Spore => self.parse_spore(),
            _ => self.parse_app(),
        }
    }

    /// Teaching diagnostic (RFC-0007 §4.8): `while`/`loop`/`break`/`continue`/`return` are not
    /// forms — and juxtaposition (`while cond …`) was never valid syntax anyway, so when one of
    /// these *unreserved* identifiers is immediately followed by an expression opener or `{`,
    /// the (inevitable) error teaches instead of confusing. Any other use stays an ordinary
    /// identifier.
    fn teach_imperative(&mut self) -> Result<(), ParseError> {
        let Tok::Ident(word) = self.cur() else {
            return Ok(());
        };
        if !matches!(
            word.as_str(),
            "while" | "loop" | "break" | "continue" | "return"
        ) {
            return Ok(());
        }
        let word = word.clone();
        let next = &self.toks[(self.i + 1).min(self.toks.len() - 1)].tok;
        let juxtaposed = matches!(
            next,
            Tok::Ident(_)
                | Tok::BinLit(_)
                | Tok::TritLit(_)
                | Tok::Int(_)
                | Tok::LBrace
                | Tok::If
                | Tok::Let
                | Tok::Match
                | Tok::For
                | Tok::Swap
        );
        if juxtaposed {
            return Err(ParseError::new(
                self.pos(),
                format!(
                    "`{word}` is not a Mycelium form — iterate by recursion or `for x in xs, \
                     acc = init => body` (bounded, total by construction; RFC-0007 §4.8)"
                ),
            ));
        }
        Ok(())
    }

    /// `for x in xs, acc = init => body` (RFC-0007 §4.8; provisional spelling, KC-2-gated).
    fn parse_for(&mut self) -> Result<Expr, ParseError> {
        self.expect(&Tok::For, "`for`")?;
        let x = self.ident()?;
        self.expect(&Tok::In, "`in` after the element binder")?;
        let xs = Box::new(self.parse_app()?);
        self.expect(&Tok::Comma, "`,` before the accumulator binding")?;
        let acc = self.ident()?;
        self.expect(&Tok::Eq, "`=` and the initial accumulator")?;
        let init = Box::new(self.parse_app()?);
        self.expect(&Tok::FatArrow, "`=>` and the fold body")?;
        let body = Box::new(self.parse_expr()?);
        Ok(Expr::For {
            x,
            xs,
            acc,
            init,
            body,
        })
    }

    fn parse_let(&mut self) -> Result<Expr, ParseError> {
        self.expect(&Tok::Let, "`let`")?;
        let name = self.ident()?;
        let ty = if self.eat(&Tok::Colon) {
            Some(self.parse_type_ref()?)
        } else {
            None
        };
        self.expect(&Tok::Eq, "`=` in the let binding")?;
        let bound = Box::new(self.parse_expr()?);
        self.expect(&Tok::In, "`in` after the let binding")?;
        let body = Box::new(self.parse_expr()?);
        Ok(Expr::Let {
            name,
            ty,
            bound,
            body,
        })
    }

    fn parse_if(&mut self) -> Result<Expr, ParseError> {
        self.expect(&Tok::If, "`if`")?;
        let cond = Box::new(self.parse_expr()?);
        self.expect(&Tok::Then, "`then`")?;
        let conseq = Box::new(self.parse_expr()?);
        self.expect(&Tok::Else, "`else`")?;
        let alt = Box::new(self.parse_expr()?);
        Ok(Expr::If { cond, conseq, alt })
    }

    fn parse_match(&mut self) -> Result<Expr, ParseError> {
        self.expect(&Tok::Match, "`match`")?;
        let scrutinee = Box::new(self.parse_expr()?);
        self.expect(&Tok::LBrace, "`{` to open the match arms")?;
        let mut arms = vec![self.parse_arm()?];
        while self.eat(&Tok::Comma) {
            if self.at(&Tok::RBrace) {
                break; // trailing comma
            }
            arms.push(self.parse_arm()?);
        }
        self.expect(
            &Tok::RBrace,
            "`}` to close the match (or `,` and another arm)",
        )?;
        Ok(Expr::Match { scrutinee, arms })
    }

    fn parse_arm(&mut self) -> Result<Arm, ParseError> {
        let pattern = self.parse_pattern()?;
        self.expect(&Tok::FatArrow, "`=>` in the match arm")?;
        let body = self.parse_expr()?;
        Ok(Arm { pattern, body })
    }

    fn parse_pattern(&mut self) -> Result<Pattern, ParseError> {
        match self.cur().clone() {
            Tok::Ident(s) if s == "_" => {
                self.bump();
                Ok(Pattern::Wildcard)
            }
            Tok::Ident(s) => {
                self.bump();
                if self.eat(&Tok::LParen) {
                    let mut subs = vec![self.parse_pattern()?];
                    while self.eat(&Tok::Comma) {
                        subs.push(self.parse_pattern()?);
                    }
                    self.expect(&Tok::RParen, "`)` to close the constructor pattern")?;
                    Ok(Pattern::Ctor(s, subs))
                } else {
                    Ok(Pattern::Ident(s))
                }
            }
            Tok::BinLit(_) | Tok::TritLit(_) | Tok::Int(_) | Tok::LBracket => {
                Ok(Pattern::Lit(self.parse_literal()?))
            }
            _ => self.err("a pattern"),
        }
    }

    fn parse_swap(&mut self) -> Result<Expr, ParseError> {
        self.expect(&Tok::Swap, "`swap`")?;
        self.expect(&Tok::LParen, "`(` after `swap`")?;
        let value = Box::new(self.parse_expr()?);
        self.expect(&Tok::Comma, "`,` before the `to:` target")?;
        self.expect(&Tok::To, "the `to:` target label")?;
        self.expect(&Tok::Colon, "`:` after `to`")?;
        let target = self.parse_type_ref()?;
        self.expect(
            &Tok::Comma,
            "`,` before the `policy:` (a swap is never silent — S1)",
        )?;
        self.expect(&Tok::Policy, "the `policy:` label (mandatory — WF2)")?;
        self.expect(&Tok::Colon, "`:` after `policy`")?;
        let policy = self.parse_path()?;
        self.expect(&Tok::RParen, "`)` to close the swap")?;
        Ok(Expr::Swap {
            value,
            target,
            policy,
        })
    }

    fn parse_wild(&mut self) -> Result<Expr, ParseError> {
        self.expect(&Tok::Wild, "`wild`")?;
        self.expect(&Tok::LBrace, "`{` to open the wild block")?;
        let body = Box::new(self.parse_expr()?);
        self.expect(&Tok::RBrace, "`}` to close the wild block")?;
        Ok(Expr::Wild(body))
    }

    fn parse_spore(&mut self) -> Result<Expr, ParseError> {
        self.expect(&Tok::Spore, "`spore`")?;
        self.expect(&Tok::LParen, "`(` after `spore`")?;
        let value = Box::new(self.parse_expr()?);
        self.expect(&Tok::RParen, "`)` to close `spore(…)`")?;
        Ok(Expr::Spore(value))
    }

    fn parse_app(&mut self) -> Result<Expr, ParseError> {
        let mut e = self.parse_primary()?;
        while self.eat(&Tok::LParen) {
            let args = self.parse_args_opt()?;
            self.expect(&Tok::RParen, "`)` to close the call")?;
            e = Expr::App {
                head: Box::new(e),
                args,
            };
        }
        if self.eat(&Tok::Colon) {
            let ty = self.parse_type_ref()?;
            e = Expr::Ascribe(Box::new(e), ty);
        }
        Ok(e)
    }

    fn parse_args_opt(&mut self) -> Result<Vec<Expr>, ParseError> {
        let mut args = Vec::new();
        if self.at(&Tok::RParen) {
            return Ok(args);
        }
        args.push(self.parse_expr()?);
        while self.eat(&Tok::Comma) {
            args.push(self.parse_expr()?);
        }
        Ok(args)
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        match self.cur() {
            Tok::BinLit(_) | Tok::TritLit(_) | Tok::Int(_) | Tok::LBracket => {
                Ok(Expr::Lit(self.parse_literal()?))
            }
            Tok::Ident(_) => Ok(Expr::Path(self.parse_path()?)),
            Tok::LParen => {
                self.bump();
                let e = self.parse_expr()?;
                self.expect(&Tok::RParen, "`)` to close the parenthesized expression")?;
                Ok(e)
            }
            _ => self.err("an expression"),
        }
    }

    fn parse_literal(&mut self) -> Result<Literal, ParseError> {
        match self.cur().clone() {
            Tok::BinLit(s) => {
                self.bump();
                Ok(Literal::Bin(s))
            }
            Tok::TritLit(s) => {
                self.bump();
                Ok(Literal::Trit(s))
            }
            Tok::Int(n) => {
                self.bump();
                Ok(Literal::Int(n))
            }
            Tok::LBracket => {
                self.bump();
                let mut elems = Vec::new();
                if !self.at(&Tok::RBracket) {
                    elems.push(self.parse_expr()?);
                    while self.eat(&Tok::Comma) {
                        elems.push(self.parse_expr()?);
                    }
                }
                self.expect(&Tok::RBracket, "`]` to close the list literal")?;
                Ok(Literal::List(elems))
            }
            _ => self.err("a literal"),
        }
    }

    fn parse_path(&mut self) -> Result<Path, ParseError> {
        let mut segs = vec![self.ident()?];
        while self.eat(&Tok::Dot) {
            segs.push(self.ident()?);
        }
        Ok(Path(segs))
    }
}
