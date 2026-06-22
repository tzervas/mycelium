//! The L1 recursive-descent parser (RFC-0006; faithful to `docs/spec/grammar/mycelium.ebnf`).
//! Hand-written, no dependencies. Every failure is an explicit [`ParseError`] with a position
//! (never a panic, never a silent accept — S5/G2). v0 covers the L1-facing core.

use crate::ast::{
    AmbientParams, Arm, BaseType, Ctor, Expr, FnDecl, FnSig, Hypha, ImplDecl, Item, Literal,
    Nodule, Paradigm, Param, Path, Pattern, Scalar, Sparsity, Strength, TraitDecl, TraitRef,
    TypeDecl, TypeParam, TypeRef,
};
use crate::error::ParseError;
use crate::lexer::lex;
use crate::token::{Pos, ScalarTok, Spanned, StrengthTok, Tok};

/// Maximum nesting depth of the expression grammar. Crafted deeply-nested input would otherwise
/// drive the recursive-descent parser (and, over the resulting AST, the typechecker / totality
/// checker / elaborator) into unbounded host-stack recursion and abort the process — `myc-check` is
/// the M-002 oracle and must return an explicit error, never crash (A4-02/B2-01). The limit is well
/// above any realistic L1 program and far below the host stack budget. Bounding the parser bounds
/// the AST depth, so the downstream passes are protected transitively.
const MAX_EXPR_DEPTH: u32 = 256;

/// Parse a complete `nodule` program from source.
pub fn parse(src: &str) -> Result<Nodule, ParseError> {
    let toks = lex(src)?;
    let mut p = Parser {
        toks,
        i: 0,
        depth: 0,
    };
    let nodule = p.parse_nodule()?;
    p.expect(&Tok::Eof, "end of input")?;
    Ok(nodule)
}

struct Parser {
    toks: Vec<Spanned>,
    i: usize,
    /// Current expression-nesting depth, bounded by [`MAX_EXPR_DEPTH`] (A4-02).
    depth: u32,
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

    // ---- separated lists (DRY, M-640) ----
    //
    // The grammar repeats two comma-separated shapes; these helpers are the single code path for
    // each, so every call site consumes byte-identical tokens and raises the identical `ParseError`
    // (the close-delimiter `expect` with its bespoke message stays at the call site). No grammar
    // change — a pure factoring of the hand-rolled loops.

    /// `one (`,` one)*` — a **non-empty** comma list, parsed *between* already-recognized delimiters
    /// (the caller consumed the opener and will `expect` the closer). Parses the first element
    /// unconditionally, then one more after each comma, stopping at the first non-comma. With
    /// `trailing_end = Some(t)`, a comma immediately followed by `t` ends the list (consumed) — the
    /// trailing-comma tolerance of `match` arms; with `None`, no trailing comma is accepted. Mirrors
    /// the bare `push(one); while eat(Comma) { … push(one) }` loop exactly. Used for constructor
    /// fields, type params, type args, sub-patterns (no trailing) and match arms (trailing).
    fn comma_separated<T>(
        &mut self,
        trailing_end: Option<&Tok>,
        mut parse_one: impl FnMut(&mut Self) -> Result<T, ParseError>,
    ) -> Result<Vec<T>, ParseError> {
        let mut items = vec![parse_one(self)?];
        while self.eat(&Tok::Comma) {
            if let Some(end) = trailing_end {
                if self.at(end) {
                    break;
                }
            }
            items.push(parse_one(self)?);
        }
        Ok(items)
    }

    /// `[ one (`,` one)* ]` — a possibly-**empty** comma list bounded by `end`, parsed *inside*
    /// already-opened delimiters (the caller consumed the opener and will `expect`/consume `end`).
    /// Equivalent to `if !at(end) { push(one); while eat(Comma) { push(one) } }`; no trailing comma.
    /// Used for value params, call args, and list-literal elements (each empty-permitting).
    fn comma_separated_until<T>(
        &mut self,
        end: &Tok,
        parse_one: impl FnMut(&mut Self) -> Result<T, ParseError>,
    ) -> Result<Vec<T>, ParseError> {
        if self.at(end) {
            return Ok(Vec::new());
        }
        self.comma_separated(None, parse_one)
    }

    /// `expect` a **leading keyword** whose diagnostic is just its own backtick-quoted spelling
    /// (`expected `let`, found …`), templating the message from one token→spelling table instead of
    /// re-spelling the keyword at each opener (M-640). Byte-identical to the prior
    /// `expect(&Tok::Let, "`let`")` form — same token consumed, same `ParseError` text. Only used at
    /// the self-naming keyword openers; sites whose message carries extra context (e.g.
    /// `"`fn` in the trait body"`) keep their bespoke [`expect`](Self::expect) call unchanged.
    fn expect_keyword(&mut self, kw: &Tok) -> Result<(), ParseError> {
        // Canonical surface spelling of each self-naming keyword opener. Total over exactly the
        // tokens the parser passes here; the `_` fallback keeps this **panic-free** (the parser
        // never panics — module invariant) by falling back to the `{:?}` form for any token outside
        // the set, so even a future miswiring is an explicit diagnostic, never a crash.
        let spelling = match kw {
            Tok::Type => "type",
            Tok::Trait => "trait",
            Tok::Fn => "fn",
            Tok::Let => "let",
            Tok::If => "if",
            Tok::Then => "then",
            Tok::Else => "else",
            Tok::Match => "match",
            Tok::For => "for",
            Tok::Swap => "swap",
            Tok::With => "with",
            Tok::Wild => "wild",
            Tok::Spore => "spore",
            Tok::Colony => "colony",
            other => return self.expect(kw, &format!("{other:?}")),
        };
        self.expect(kw, &format!("`{spelling}`"))
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

    fn parse_nodule(&mut self) -> Result<Nodule, ParseError> {
        self.expect(&Tok::Nodule, "a `nodule` header to open the program")?;
        let path = self.parse_path()?;
        let mut items = Vec::new();
        while !self.at(&Tok::Eof) {
            items.push(self.parse_item()?);
        }
        Ok(Nodule { path, items })
    }

    fn parse_item(&mut self) -> Result<Item, ParseError> {
        match self.cur() {
            Tok::Use => {
                self.bump();
                Ok(Item::Use(self.parse_path()?))
            }
            Tok::Default => {
                self.bump();
                self.expect(&Tok::Paradigm, "`paradigm` after `default` (RFC-0012 §4.2)")?;
                Ok(Item::Default(self.parse_paradigm()?))
            }
            Tok::Type => self.parse_type_decl().map(Item::Type),
            Tok::Trait => self.parse_trait_decl().map(Item::Trait),
            // M-659 / RFC-0019 §4.1: `impl Trait<args> for T { fn … }` (the trait-instance
            // production). `impl` was reserved by M-658 (RFC-0007 §12.2); this is the production that
            // consumes it.
            Tok::Impl => self.parse_impl_decl().map(Item::Impl),
            Tok::Fn | Tok::Thaw => self.parse_fn_decl().map(Item::Fn),
            Tok::Matured => Err(ParseError::new(
                self.pos(),
                "maturation is declared per `nodule`/`phylum` in the header \
                     (`// @matured: true`) or per program in the manifest — RFC-0017 §4.1; \
                     to keep one definition interpreted inside a matured scope use `thaw fn`"
                    .to_owned(),
            )),
            // M-666 / RFC-0008 §4.7: `colony` and `hypha` are now **active**, but as *expressions*,
            // not top-level items — they live inside a `fn` body. Teaching diagnostics point there
            // (never a silent accept, G2).
            Tok::Colony => Err(ParseError::new(
                self.pos(),
                "`colony { … }` is an expression (a structured-concurrency scope; RFC-0008 §4.7), \
                 not a top-level item — write it inside a `fn` body"
                    .to_owned(),
            )),
            Tok::Hypha => Err(ParseError::new(
                self.pos(),
                "`hypha <expr>` spawns a concurrent task and is only valid inside a `colony { … }` \
                 block (RFC-0008 §4.7 RT7 — an orphan hypha is not expressible), not at item position"
                    .to_owned(),
            )),
            // DN-03 §4 / RFC-0008 §4.5: the remaining runtime-vocabulary reserved words. They lex as
            // keywords (never silent identifiers, G2) but no L1 construct consumes them yet — teaching
            // diagnostic, never a silent accept. (`hypha`/`colony` left the set with M-666.)
            t @ (Tok::Fuse
            | Tok::Mesh
            | Tok::Graft
            | Tok::Cyst
            | Tok::Xloc
            | Tok::Forage
            | Tok::Backbone
            | Tok::Tier
            | Tok::Reclaim) => Err(ParseError::new(
                self.pos(),
                format!(
                    "`{word}` is reserved for the runtime model (RFC-0008), not yet active — \
                     it cannot open a program or be used as an identifier at this language version",
                    word = runtime_keyword_spelling(t)
                ),
            )),
            _ => self.err(
                "a top-level item (`use`, `default paradigm`, `type`, `trait`, `impl`, `fn`, or \
                 `thaw fn`)",
            ),
        }
    }

    /// A bare paradigm tag (`Binary|Ternary|Dense|VSA`) for an ambient declaration (RFC-0012 §4.2).
    fn parse_paradigm(&mut self) -> Result<Paradigm, ParseError> {
        let p = match self.cur() {
            Tok::Binary => Paradigm::Binary,
            Tok::Ternary => Paradigm::Ternary,
            Tok::Dense => Paradigm::Dense,
            Tok::Vsa => Paradigm::Vsa,
            _ => return self.err("a paradigm (`Binary|Ternary|Dense|VSA`)"),
        };
        self.bump();
        Ok(p)
    }

    fn parse_type_decl(&mut self) -> Result<TypeDecl, ParseError> {
        self.expect_keyword(&Tok::Type)?;
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
            fields = self.comma_separated(None, Self::parse_type_ref)?;
            self.expect(&Tok::RParen, "`)` to close the constructor fields")?;
        }
        Ok(Ctor { name, fields })
    }

    fn parse_trait_decl(&mut self) -> Result<TraitDecl, ParseError> {
        self.expect_keyword(&Tok::Trait)?;
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
        let thaw = self.eat(&Tok::Thaw);
        self.expect(&Tok::Fn, "`fn`")?;
        let sig = self.parse_sig_tail()?;
        self.expect(&Tok::Eq, "`=` before the function body")?;
        let body = self.parse_expr()?;
        Ok(FnDecl { thaw, sig, body })
    }

    /// The shared `name <params>? ( value_params? ) -> ret !{effects}?` tail of a signature. A
    /// function's type-parameters may carry **trait bounds** (`<T: Cmp + Ord<T>>`; RFC-0019 §4.1) —
    /// the dictionary site — so this uses
    /// [`parse_type_params_bounded`](Self::parse_type_params_bounded). The optional
    /// `!{ eff1, eff2 }` **effect annotation** (RFC-0014 §3.4; M-660) follows the return type; absent
    /// ⇒ the empty (pure) effect set.
    fn parse_sig_tail(&mut self) -> Result<FnSig, ParseError> {
        let name = self.ident()?;
        let params = self.parse_type_params_bounded()?;
        self.expect(&Tok::LParen, "`(` to open the parameter list")?;
        let value_params = self.parse_params_opt()?;
        self.expect(&Tok::RParen, "`)` to close the parameter list")?;
        self.expect(&Tok::Arrow, "`->` and a result type")?;
        let ret = self.parse_type_ref()?;
        let effects = self.parse_effects_opt()?;
        Ok(FnSig {
            name,
            params,
            value_params,
            ret,
            effects,
        })
    }

    /// `!{ eff (, eff)* }?` — the optional **effect annotation** after a signature's return type
    /// (RFC-0014 §3.4/§4.5 I3; M-660). When the next token is `!`, consume `!{` … `}` and parse a
    /// comma-separated list of effect **names** (plain identifiers — the closed kernel kinds
    /// `retry|alloc|io|cascade|time` plus user `Named` effects; RFC-0014 §4.5); the empty set `!{}`
    /// is allowed (an explicit, written "pure" annotation). When there is no `!`, return `vec![]`
    /// (the implicit pure default — RFC-0014 I5). A **duplicate** effect name in one annotation is an
    /// explicit refusal (never a silent dedup — G2): a repeated effect is a written redundancy the
    /// author should fix, not something the parser quietly collapses.
    fn parse_effects_opt(&mut self) -> Result<Vec<String>, ParseError> {
        if !self.eat(&Tok::Bang) {
            return Ok(Vec::new());
        }
        self.expect(
            &Tok::LBrace,
            "`{` after `!` to open the effect set (RFC-0014 §3.4)",
        )?;
        // The empty written set `!{}` is valid (an explicit "declares no effects").
        let effects = if self.at(&Tok::RBrace) {
            Vec::new()
        } else {
            self.comma_separated(None, Self::ident)?
        };
        self.expect(&Tok::RBrace, "`}` to close the effect set")?;
        if let Some(dup) = first_duplicate_str(&effects) {
            return Err(ParseError::new(
                self.pos(),
                format!(
                    "duplicate effect `{dup}` in the effect annotation — list each declared effect \
                     once (RFC-0014 §4.5; a repeated effect is a never-silent refusal, not a silent \
                     dedup)"
                ),
            ));
        }
        Ok(effects)
    }

    fn parse_params_opt(&mut self) -> Result<Vec<Param>, ParseError> {
        self.comma_separated_until(&Tok::RParen, |p| {
            let name = p.ident()?;
            p.expect(&Tok::Colon, "`:` and the parameter type")?;
            let ty = p.parse_type_ref()?;
            Ok(Param { name, ty })
        })
    }

    /// `< name (, name)* >?` — **unbounded** type-parameter names, for `type`/`trait` declarations
    /// (stage-1: data/trait type-params are unbounded abstractions — RFC-0019 §4.1 / RFC-0007 §12.1).
    /// A bound (`<T: Cmp>`) here is an **explicit refusal** (deferred to a later stage), never
    /// silently dropped — bounds belong only on function type-params (the dictionary site).
    fn parse_type_params_opt(&mut self) -> Result<Vec<String>, ParseError> {
        let mut params = Vec::new();
        if self.eat(&Tok::LAngle) {
            params = self.comma_separated(None, |p| {
                let name = p.ident()?;
                if p.at(&Tok::Colon) {
                    return Err(ParseError::new(
                        p.pos(),
                        "bounds on `type`/`trait` type-parameters are deferred in stage-1 \
                         (RFC-0019 §4.1 — bounds live only on function type-parameters, the \
                         dictionary site); write the bound on the bounded `fn` instead"
                            .to_owned(),
                    ));
                }
                Ok(name)
            })?;
            self.expect(&Tok::RAngle, "`>` to close the type parameters")?;
        }
        Ok(params)
    }

    /// `< type_param (, type_param)* >?` where `type_param ::= Ident (':' bound)?` — **bounded**
    /// type-parameters for **functions** (RFC-0019 §4.1). An unbounded `T` yields
    /// `TypeParam { bounds: [] }` (the §11 identity, so every v0 program still parses).
    fn parse_type_params_bounded(&mut self) -> Result<Vec<TypeParam>, ParseError> {
        let mut params = Vec::new();
        if self.eat(&Tok::LAngle) {
            params = self.comma_separated(None, Self::parse_type_param)?;
            self.expect(&Tok::RAngle, "`>` to close the type parameters")?;
        }
        Ok(params)
    }

    /// One bounded type-parameter `Ident (':' bound)?` (RFC-0019 §4.1).
    fn parse_type_param(&mut self) -> Result<TypeParam, ParseError> {
        let name = self.ident()?;
        let bounds = if self.eat(&Tok::Colon) {
            self.parse_bound()?
        } else {
            Vec::new()
        };
        Ok(TypeParam { name, bounds })
    }

    /// A trait bound `Ident type_args? ('+' Ident type_args?)*` — one or more trait references
    /// (RFC-0019 §4.1 `bound`). Reuses the existing type-argument parser for each trait's `<…>`.
    fn parse_bound(&mut self) -> Result<Vec<TraitRef>, ParseError> {
        let mut bounds = vec![self.parse_trait_ref()?];
        while self.eat(&Tok::Plus) {
            bounds.push(self.parse_trait_ref()?);
        }
        Ok(bounds)
    }

    /// One trait reference in a bound — `Cmp` or `Cmp<Binary{8}>` (RFC-0019 §4.1).
    fn parse_trait_ref(&mut self) -> Result<TraitRef, ParseError> {
        let name = self.ident()?;
        let args = self.parse_type_args_opt()?;
        Ok(TraitRef { name, args })
    }

    /// `impl Ident type_args? 'for' type_ref '{' fn_decl* '}'` — a trait instance (RFC-0019 §4.1;
    /// RFC-0007 §12.1). The `<…>` are the trait's **type arguments** (concrete `TypeRef`s, reusing
    /// the existing type-arg parser), not parameter names; the methods are full `fn … = body` defs.
    fn parse_impl_decl(&mut self) -> Result<ImplDecl, ParseError> {
        self.expect(&Tok::Impl, "`impl`")?;
        let trait_name = self.ident()?;
        let trait_args = self.parse_type_args_opt()?;
        self.expect(
            &Tok::For,
            "`for` after the trait in an `impl` (RFC-0019 §4.1)",
        )?;
        let for_ty = self.parse_type_ref()?;
        self.expect(&Tok::LBrace, "`{` to open the `impl` body")?;
        let mut methods = Vec::new();
        while !self.at(&Tok::RBrace) {
            methods.push(self.parse_fn_decl()?);
        }
        self.expect(&Tok::RBrace, "`}` to close the `impl` body")?;
        Ok(ImplDecl {
            trait_name,
            trait_args,
            for_ty,
            methods,
        })
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
            // A paradigm-less repr `{ … }` (RFC-0012 §4.2): the paradigm is supplied later by the
            // enclosing ambient; only the size/shape is written here. The shape (single size vs
            // Dense `{N, scalar}` vs VSA `{model, dim, sparsity}`) is disambiguated by lookahead;
            // whether it *fits* the ambient paradigm is the resolution pass's never-silent check.
            Tok::LBrace => self.parse_ambient_repr().map(BaseType::Ambient),
            _ => self.err("a type"),
        }
    }

    /// Parse a paradigm-less repr's params `{ … }` into [`AmbientParams`] (RFC-0012 §4.2). The
    /// leading token disambiguates: an `Int` opens a size (`{N}`) or a Dense shape (`{N, scalar}`);
    /// an `Ident` opens a VSA shape (`{model, dim, sparsity}`).
    fn parse_ambient_repr(&mut self) -> Result<AmbientParams, ParseError> {
        self.expect(&Tok::LBrace, "`{` to open the paradigm-less repr")?;
        let params = match self.cur() {
            Tok::Int(_) => {
                let n = self.u32_lit()?;
                if self.eat(&Tok::Comma) {
                    let scalar = self.parse_scalar()?;
                    AmbientParams::Dense(n, scalar)
                } else {
                    AmbientParams::Size(n)
                }
            }
            Tok::Ident(_) => {
                let model = self.ident()?;
                self.expect(&Tok::Comma, "`,` after the VSA model")?;
                let dim = self.u32_lit()?;
                self.expect(&Tok::Comma, "`,` before the sparsity")?;
                let sparsity = self.parse_sparsity()?;
                AmbientParams::Vsa {
                    model,
                    dim,
                    sparsity,
                }
            }
            _ => {
                return self
                    .err("a paradigm-less repr param (a size `{N}`, `{N, scalar}`, or VSA shape)")
            }
        };
        self.expect(&Tok::RBrace, "`}` to close the paradigm-less repr")?;
        Ok(params)
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
            args = self.comma_separated(None, Self::parse_type_ref)?;
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

    /// Depth-guarded entry to the expression grammar: refuses to recurse past [`MAX_EXPR_DEPTH`]
    /// with an explicit error rather than overflowing the host stack on crafted nesting (A4-02).
    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        self.depth += 1;
        if self.depth > MAX_EXPR_DEPTH {
            self.depth -= 1;
            return Err(ParseError::new(
                self.pos(),
                format!("expression nests deeper than the limit of {MAX_EXPR_DEPTH} — refusing to recurse"),
            ));
        }
        let r = self.parse_expr_inner();
        self.depth -= 1;
        r
    }

    fn parse_expr_inner(&mut self) -> Result<Expr, ParseError> {
        self.teach_imperative()?;
        // M-666 / RFC-0008 §4.7: a bare `hypha <expr>` at expression position is only valid *inside*
        // a `colony { … }` block (RT7 — an orphan hypha is not expressible); `parse_colony` consumes
        // the `hypha` keywords in the block body, so reaching one here means it is unscoped. Explicit
        // teaching diagnostic, never a silent accept (G2).
        if self.at(&Tok::Hypha) {
            return Err(ParseError::new(
                self.pos(),
                "`hypha <expr>` spawns a concurrent task and is only valid inside a `colony { … }` \
                 block (RFC-0008 §4.7 RT7 — an orphan hypha is not expressible)"
                    .to_owned(),
            ));
        }
        // DN-03 §4 / RFC-0008 §4.5: the remaining runtime-vocabulary reserved words produce a
        // teaching diagnostic at expression position (never a silent accept, G2). (`hypha`/`colony`
        // left the reserved set with M-666 — `colony` is dispatched below; `hypha` is handled above.)
        if let t @ (Tok::Fuse
        | Tok::Mesh
        | Tok::Graft
        | Tok::Cyst
        | Tok::Xloc
        | Tok::Forage
        | Tok::Backbone
        | Tok::Tier
        | Tok::Reclaim) = self.cur()
        {
            return Err(ParseError::new(
                self.pos(),
                format!(
                    "`{word}` is reserved for the runtime model (RFC-0008), not yet active — \
                     it cannot open a program or be used as an identifier at this language version",
                    word = runtime_keyword_spelling(t)
                ),
            ));
        }
        match self.cur() {
            Tok::Let => self.parse_let(),
            Tok::If => self.parse_if(),
            Tok::Match => self.parse_match(),
            Tok::For => self.parse_for(),
            Tok::Swap => self.parse_swap(),
            Tok::With => self.parse_with_paradigm(),
            Tok::Wild => self.parse_wild(),
            Tok::Spore => self.parse_spore(),
            Tok::Colony => self.parse_colony(),
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

    /// `for x in xs, acc = init => body` (RFC-0007 §4.8; spelling adopted at r3).
    fn parse_for(&mut self) -> Result<Expr, ParseError> {
        self.expect_keyword(&Tok::For)?;
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
        self.expect_keyword(&Tok::Let)?;
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
        self.expect_keyword(&Tok::If)?;
        let cond = Box::new(self.parse_expr()?);
        self.expect_keyword(&Tok::Then)?;
        let conseq = Box::new(self.parse_expr()?);
        self.expect_keyword(&Tok::Else)?;
        let alt = Box::new(self.parse_expr()?);
        Ok(Expr::If { cond, conseq, alt })
    }

    fn parse_match(&mut self) -> Result<Expr, ParseError> {
        self.expect_keyword(&Tok::Match)?;
        let scrutinee = Box::new(self.parse_expr()?);
        self.expect(&Tok::LBrace, "`{` to open the match arms")?;
        // Non-empty (≥ 1 arm), with a trailing comma before `}` tolerated.
        let arms = self.comma_separated(Some(&Tok::RBrace), Self::parse_arm)?;
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
                    let subs = self.comma_separated(None, Self::parse_pattern)?;
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
        self.expect_keyword(&Tok::Swap)?;
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

    /// `with paradigm P { e }` — a block-scope ambient override (RFC-0012 §4.4). Not a conversion:
    /// the resolution pass fills the interior tags and strips the block; an unbridged cross-paradigm
    /// edge is a never-silent `MissingConversion`.
    fn parse_with_paradigm(&mut self) -> Result<Expr, ParseError> {
        self.expect_keyword(&Tok::With)?;
        self.expect(&Tok::Paradigm, "`paradigm` after `with` (RFC-0012 §4.4)")?;
        let paradigm = self.parse_paradigm()?;
        self.expect(&Tok::LBrace, "`{` to open the `with paradigm` block")?;
        let body = Box::new(self.parse_expr()?);
        self.expect(&Tok::RBrace, "`}` to close the `with paradigm` block")?;
        Ok(Expr::WithParadigm { paradigm, body })
    }

    fn parse_wild(&mut self) -> Result<Expr, ParseError> {
        self.expect_keyword(&Tok::Wild)?;
        self.expect(&Tok::LBrace, "`{` to open the wild block")?;
        let body = Box::new(self.parse_expr()?);
        self.expect(&Tok::RBrace, "`}` to close the wild block")?;
        Ok(Expr::Wild(body))
    }

    fn parse_spore(&mut self) -> Result<Expr, ParseError> {
        self.expect_keyword(&Tok::Spore)?;
        self.expect(&Tok::LParen, "`(` after `spore`")?;
        let value = Box::new(self.parse_expr()?);
        self.expect(&Tok::RParen, "`)` to close `spore(…)`")?;
        Ok(Expr::Spore(value))
    }

    /// `colony { hypha e1, hypha e2, … }` — the structured-concurrency scope (RFC-0008 §4.7; DN-06
    /// §1.3). The block body is a **non-empty** comma-separated list of `hypha <expr>` spawns; an
    /// empty `colony { }` is an explicit error (a colony with no hyphae is meaningless — RT7 names a
    /// *grouping of active hyphae*). Each `hypha` keyword opens one spawn whose body is an
    /// application/expression over immutable values; a trailing comma before `}` is tolerated (the
    /// `match`-arm convention). Deterministic R1 fragment only (RFC-0008 §4.6 R1) — the
    /// arbitration/placement RT3 constructs are separate, later work.
    fn parse_colony(&mut self) -> Result<Expr, ParseError> {
        self.expect_keyword(&Tok::Colony)?;
        self.expect(&Tok::LBrace, "`{` to open the `colony` block")?;
        // Non-empty (≥ 1 hypha), trailing comma before `}` tolerated.
        let hyphae = self.comma_separated(Some(&Tok::RBrace), Self::parse_hypha)?;
        self.expect(
            &Tok::RBrace,
            "`}` to close the `colony` (or `,` and another `hypha`)",
        )?;
        Ok(Expr::Colony(hyphae))
    }

    /// One `hypha <expr>` spawn inside a [`parse_colony`] body. The `hypha` keyword is mandatory
    /// (RT7: every concurrent unit is named — a bare body would be ambiguous with a value), and its
    /// computation is parsed as an `app_expr` (a call like `compute(x)` — the issue's canonical form),
    /// matching `for`'s use of `app_expr` for its bounded sub-expressions.
    fn parse_hypha(&mut self) -> Result<Hypha, ParseError> {
        self.expect(
            &Tok::Hypha,
            "`hypha` to open a concurrent task in the `colony` block (RFC-0008 §4.7)",
        )?;
        let body = self.parse_app()?;
        Ok(Hypha { body })
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
        self.comma_separated_until(&Tok::RParen, Self::parse_expr)
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
                let elems = self.comma_separated_until(&Tok::RBracket, Self::parse_expr)?;
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

/// The first value appearing more than once in `xs` (left to right), if any. Used by the effect
/// annotation parser to reject a duplicate effect name explicitly (M-660; G2 — never a silent
/// dedup). A small, allocation-light scan (effect sets are short); mirrors the checker's
/// `first_duplicate` without coupling the two modules.
fn first_duplicate_str(xs: &[String]) -> Option<&String> {
    let mut seen = std::collections::BTreeSet::new();
    xs.iter().find(|x| !seen.insert((*x).as_str()))
}

/// Return the surface spelling for a DN-03 §4 runtime-vocabulary reserved keyword token.
/// Used in teaching diagnostics so the error message names the actual word, not the enum variant.
/// Total over exactly the runtime-vocabulary tokens; the `_` arm is unreachable in practice
/// (callers only pass one of the ten runtime-vocab arms) but keeps this panic-free (G2).
fn runtime_keyword_spelling(tok: &Tok) -> &'static str {
    match tok {
        Tok::Hypha => "hypha",
        Tok::Fuse => "fuse",
        Tok::Mesh => "mesh",
        Tok::Graft => "graft",
        Tok::Cyst => "cyst",
        Tok::Xloc => "xloc",
        Tok::Forage => "forage",
        Tok::Backbone => "backbone",
        Tok::Tier => "tier",
        Tok::Reclaim => "reclaim",
        _ => "<runtime-keyword>",
    }
}

#[cfg(test)]
mod tests {
    //! Behavioral tests for the M-640 separated-list / keyword `expect` factoring. They drive the
    //! private helpers through the public [`parse`] entry on representative grammar so the folded
    //! call sites are exercised end-to-end (empty / single / multi lists, the match-arm trailing
    //! comma, and that bare lists reject a trailing comma) — pinning byte-identical behavior.
    use super::*;
    use crate::ast::{Expr, Item, Literal};

    fn fn_body(src: &str) -> Expr {
        let n = parse(src).expect("parses");
        n.items
            .into_iter()
            .find_map(|i| match i {
                Item::Fn(fd) => Some(fd.body),
                _ => None,
            })
            .expect("a fn item")
    }

    #[test]
    fn empty_list_literal_parses_to_no_elems() {
        // `comma_separated_until(RBracket)` empty path.
        let Expr::Lit(Literal::List(elems)) = fn_body("nodule d\nfn main() -> Binary{1} = []")
        else {
            panic!("expected a list literal");
        };
        assert!(elems.is_empty());
    }

    #[test]
    fn single_and_multi_element_list_literals() {
        let one = fn_body("nodule d\nfn main() -> Binary{1} = [0b0]");
        let Expr::Lit(Literal::List(e1)) = one else {
            panic!("list")
        };
        assert_eq!(e1.len(), 1);
        let many = fn_body("nodule d\nfn main() -> Binary{1} = [0b0, 0b1, 0b0]");
        let Expr::Lit(Literal::List(e3)) = many else {
            panic!("list")
        };
        assert_eq!(e3.len(), 3);
    }

    #[test]
    fn call_args_empty_single_multi() {
        // `comma_separated_until(RParen)` for application args.
        let zero = fn_body("nodule d\nfn main() -> Binary{1} = f()");
        let Expr::App { args, .. } = zero else {
            panic!("app")
        };
        assert_eq!(args.len(), 0);
        let two = fn_body("nodule d\nfn main() -> Binary{1} = f(0b0, 0b1)");
        let Expr::App { args, .. } = two else {
            panic!("app")
        };
        assert_eq!(args.len(), 2);
    }

    #[test]
    fn ctor_fields_and_type_params_and_args() {
        // Constructor fields (`comma_separated` after `(`), type params/args (`<…>`).
        let n = parse(
            "nodule d\ntype Pair<A, B> = MkPair(A, B)\n\
             fn id(x: Pair<Binary{1}, Binary{1}>) -> Pair<Binary{1}, Binary{1}> = x",
        )
        .expect("parses");
        let Item::Type(td) = &n.items[0] else {
            panic!("type decl")
        };
        assert_eq!(td.params, vec!["A".to_owned(), "B".to_owned()]);
        assert_eq!(td.ctors.len(), 1);
        assert_eq!(td.ctors[0].fields.len(), 2); // two ctor fields
    }

    #[test]
    fn value_params_empty_and_nonempty() {
        let zero = parse("nodule d\nfn main() -> Binary{1} = 0b0").expect("parses");
        let Item::Fn(fd) = &zero.items[0] else {
            panic!("fn")
        };
        assert_eq!(fd.sig.value_params.len(), 0);
        let two =
            parse("nodule d\nfn g(a: Binary{1}, b: Binary{1}) -> Binary{1} = a").expect("parses");
        let Item::Fn(fd) = &two.items[0] else {
            panic!("fn")
        };
        assert_eq!(fd.sig.value_params.len(), 2);
    }

    #[test]
    fn match_arms_tolerate_a_trailing_comma() {
        // `comma_separated(Some(RBrace))` trailing-comma path — same arm count with or without it.
        let with = fn_body(
            "nodule d\ntype B = F | T\nfn m(x: B) -> Binary{1} = match x { F => 0b0, T => 0b1, }",
        );
        let Expr::Match { arms, .. } = with else {
            panic!("match")
        };
        assert_eq!(arms.len(), 2);
        let without = fn_body(
            "nodule d\ntype B = F | T\nfn m(x: B) -> Binary{1} = match x { F => 0b0, T => 0b1 }",
        );
        let Expr::Match { arms, .. } = without else {
            panic!("match")
        };
        assert_eq!(arms.len(), 2);
    }

    #[test]
    fn empty_match_is_still_an_explicit_error() {
        // The non-empty invariant of match arms must survive the factoring: `match x { }` parses the
        // first arm and fails on the pattern — never silently an empty arm list.
        let err = parse("nodule d\ntype B = F\nfn m(x: B) -> Binary{1} = match x { }")
            .expect_err("empty match must be rejected");
        assert!(err.message.contains("a pattern"), "{}", err.message);
    }

    #[test]
    fn a_bare_list_rejects_a_trailing_comma() {
        // Constructor fields take no trailing comma (`comma_separated(None)`): a dangling `,` makes
        // the helper try to parse another field and fail explicitly — behavior unchanged by M-640.
        let err = parse("nodule d\ntype T = C(Binary{1},)")
            .expect_err("trailing comma in ctor fields must be rejected");
        assert!(err.message.contains("expected a type"), "{}", err.message);
    }

    #[test]
    fn keyword_opener_diagnostic_is_the_backtick_spelling() {
        // `expect_keyword` must reproduce the exact `` `let` `` (etc.) message of the old inline
        // form. A `let` body that is truncated right where a keyword opener is required surfaces it.
        let err = parse("nodule d\nfn main() -> Binary{1} = if 0b0 then 0b1 els 0b0")
            .expect_err("malformed if must be rejected");
        // `els` is an identifier where `else` is required.
        assert!(err.message.contains("`else`"), "{}", err.message);
    }

    // --- M-659 / RFC-0019 §4.1: `impl` decls + bounded type-params parse ---

    #[test]
    fn an_impl_decl_parses_with_trait_args_for_type_and_methods() {
        let n = parse(
            "nodule d\ntrait Cmp<A> { fn cmp(a: A, b: A) -> Binary{2} }\n\
             impl Cmp<Binary{8}> for Binary{8} \
             { fn cmp(a: Binary{8}, b: Binary{8}) -> Binary{2} = 0b00 }",
        )
        .expect("an impl parses");
        let Item::Impl(id) = n
            .items
            .iter()
            .find(|i| matches!(i, Item::Impl(_)))
            .expect("an impl item")
        else {
            panic!("impl");
        };
        assert_eq!(id.trait_name, "Cmp");
        assert_eq!(id.trait_args.len(), 1); // `<Binary{8}>`
        assert_eq!(id.methods.len(), 1);
        assert_eq!(id.methods[0].sig.name, "cmp");
    }

    #[test]
    fn an_impl_without_for_is_an_explicit_error() {
        let err = parse("nodule d\nimpl Cmp<Binary{8}> Binary{8} { }")
            .expect_err("impl missing `for` must be rejected");
        assert!(err.message.contains("`for`"), "{}", err.message);
    }

    #[test]
    fn a_bounded_fn_type_param_parses_with_a_self_bound_and_a_plus_list() {
        // `<T: Cmp>` (single self-bound) and `<T: A + B<T>>` (a `+`-list with type-args) both parse.
        let n = parse(
            "nodule d\nfn f<T: Cmp>(x: T) -> T = x\n\
             fn g<T: A + B<T>>(x: T) -> T = x",
        )
        .expect("bounded type-params parse");
        let Item::Fn(f) = &n.items[0] else {
            panic!("fn")
        };
        assert_eq!(f.sig.params.len(), 1);
        assert_eq!(f.sig.params[0].name, "T");
        assert_eq!(f.sig.params[0].bounds.len(), 1);
        assert_eq!(f.sig.params[0].bounds[0].name, "Cmp");
        let Item::Fn(g) = &n.items[1] else {
            panic!("fn")
        };
        assert_eq!(g.sig.params[0].bounds.len(), 2); // A + B<T>
        assert_eq!(g.sig.params[0].bounds[1].name, "B");
        assert_eq!(g.sig.params[0].bounds[1].args.len(), 1); // B<T>
    }

    #[test]
    fn an_unbounded_fn_type_param_still_parses_the_identity_case() {
        // The §11 identity: `<A>` with no bound is `TypeParam { bounds: [] }` — every v0 program
        // that parsed before this extension still parses.
        let n = parse("nodule d\nfn id<A>(x: A) -> A = x").expect("unbounded parses");
        let Item::Fn(f) = &n.items[0] else {
            panic!("fn")
        };
        assert_eq!(f.sig.params.len(), 1);
        assert!(f.sig.params[0].bounds.is_empty());
    }

    #[test]
    fn a_bound_on_a_type_decl_param_is_an_explicit_parse_refusal() {
        // Stage-1: bounds live only on fn type-params. A bound on a `type` param is rejected, never
        // silently dropped (G2). (Conformance reject/15 pins this at the corpus level too.)
        let err = parse("nodule d\ntype Box<A: Cmp> = Wrap(A)")
            .expect_err("a bound on a type-decl param must be rejected");
        assert!(err.message.contains("deferred"), "{}", err.message);
    }
}
