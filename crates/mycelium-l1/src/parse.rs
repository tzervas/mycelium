//! The L1 recursive-descent parser (RFC-0006; faithful to `docs/spec/grammar/mycelium.ebnf`).
//! Hand-written, no dependencies. Every failure is an explicit [`ParseError`] with a position
//! (never a panic, never a silent accept — S5/G2). v0 covers the L1-facing core.

use crate::ast::{
    AmbientParams, Arm, BaseType, Ctor, Expr, FnDecl, FnSig, Hypha, ImplDecl, Item, Literal,
    Nodule, Paradigm, Param, Path, Pattern, Phylum, Scalar, Sparsity, Strength, TraitDecl,
    TraitRef, TypeDecl, TypeParam, TypeRef, UsePath, Vis,
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

/// Parse a complete **single-`nodule`** program from source — the v0 entry point, unchanged by the
/// phylum work (M-662). A bare `nodule <path> <item>*` parses to a [`Nodule`]; trailing content (a
/// second `nodule`, a `phylum` header) is an explicit error here. Multi-nodule / phylum-headed source
/// uses [`parse_phylum`]; a [`Nodule`] *is* a phylum-of-one ([`Phylum::of_one`]).
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

/// Parse a complete **phylum** program (M-662; RFC-0006 §4.3): an optional `phylum <path>` header
/// followed by **one-or-more** `nodule` blocks, into a [`Phylum`]. A header-less single `nodule`
/// parses to a phylum-of-one (`path: None`) — so `parse_phylum` is a strict superset of [`parse`]:
/// every program [`parse`] accepts, `parse_phylum` accepts identically (as a phylum-of-one), and it
/// additionally accepts a `phylum` header and multiple nodules. A `phylum` header with **zero**
/// nodules is an explicit error (a phylum groups nodules — there must be at least one).
///
/// # Errors
/// Returns a [`ParseError`] for any malformed header, item, or a `phylum` header followed by no
/// `nodule` (never a panic, never a silent accept — S5/G2).
pub fn parse_phylum(src: &str) -> Result<Phylum, ParseError> {
    let toks = lex(src)?;
    let mut p = Parser {
        toks,
        i: 0,
        depth: 0,
    };
    let phylum = p.parse_phylum()?;
    p.expect(&Tok::Eof, "end of input")?;
    Ok(phylum)
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

    /// `phylum_header? nodule+` — a whole phylum program (M-662; RFC-0006 §4.3). An optional
    /// `phylum <path>` header (the library-scale grouping; DN-06) precedes **one-or-more** `nodule`
    /// blocks. `phylum` is a *grouping*, not a container — no `phylum { … }` block; the nodules follow
    /// the header at top level. A header with no following `nodule` is an explicit error.
    fn parse_phylum(&mut self) -> Result<Phylum, ParseError> {
        // Optional `phylum <path>` header. `phylum` activates as a header keyword here (M-662); it was
        // reserved-not-active before. It carries a dotted path and opens no block.
        let path = if self.eat(&Tok::Phylum) {
            Some(self.parse_path()?)
        } else {
            None
        };
        let mut nodules = Vec::new();
        // One-or-more `nodule` blocks. The first must be present (a phylum, headed or not, groups at
        // least one nodule); each `parse_nodule` consumes its items up to the next `nodule`/EOF.
        if !self.at(&Tok::Nodule) {
            return Err(ParseError::new(
                self.pos(),
                if path.is_some() {
                    "a `phylum` header must be followed by at least one `nodule` block \
                     (a phylum groups nodules — RFC-0006 §4.3)"
                        .to_owned()
                } else {
                    "expected a `nodule` header to open the program".to_owned()
                },
            ));
        }
        while self.at(&Tok::Nodule) {
            nodules.push(self.parse_nodule()?);
        }
        Ok(Phylum { path, nodules })
    }

    /// `nodule <path> @std-sys? <item>*` — one nodule block (RFC-0006 §4.3). An optional
    /// **`@std-sys`** marker after the path (M-661; RFC-0016 §8-Q6) tags the nodule as the audited
    /// FFI-floor context: only a `@std-sys` nodule may contain a `wild` block (the checker enforces
    /// this — a `wild` elsewhere is a hard refusal, never silent — G2). The marker is lexed atomically
    /// as [`Tok::AtStdSys`] (so `@std-sys`'s `-` is not a lex error); absent ⇒ a normal nodule.
    ///
    /// Items run until the **next `nodule` header or EOF** (M-662): in a multi-nodule phylum the items
    /// of one nodule end where the next `nodule` begins. For a single-nodule program ([`parse`]) the
    /// loop simply runs to EOF, exactly as before — backward-compatible (a `nodule` *is* a
    /// phylum-of-one).
    fn parse_nodule(&mut self) -> Result<Nodule, ParseError> {
        self.expect(&Tok::Nodule, "a `nodule` header to open the program")?;
        let path = self.parse_path()?;
        // Optional `@std-sys` FFI-floor header marker (M-661). It is the audited-FFI *context* gate;
        // it carries no further syntax (no `: true`/`: false` — its mere presence is the attribute).
        let std_sys = self.eat(&Tok::AtStdSys);
        let mut items = Vec::new();
        // Stop at the next `nodule` (the start of a sibling nodule in a phylum) or EOF.
        while !self.at(&Tok::Eof) && !self.at(&Tok::Nodule) {
            items.push(self.parse_item()?);
        }
        Ok(Nodule {
            path,
            std_sys,
            items,
        })
    }

    fn parse_item(&mut self) -> Result<Item, ParseError> {
        // A leading `pub` marks a top-level `fn`/`trait`/`type` (or `thaw fn`) as cross-nodule
        // **exported** (M-662). It is only valid before one of those declarations — a `pub` before
        // `use`/`default`/`impl` (or anything else) is an explicit refusal, never a silent accept
        // (G2). `impl`/`default`/`use` are not part of the `pub` namespace (a `use` imports, it does
        // not re-export). The `pub` is consumed here and threaded into the declaration's `vis`.
        if self.at(&Tok::Pub) {
            return self.parse_pub_item();
        }
        match self.cur() {
            Tok::Use => self.parse_use().map(Item::Use),
            Tok::Default => {
                self.bump();
                self.expect(&Tok::Paradigm, "`paradigm` after `default` (RFC-0012 §4.2)")?;
                Ok(Item::Default(self.parse_paradigm()?))
            }
            Tok::Type => self.parse_type_decl(Vis::Private).map(Item::Type),
            Tok::Trait => self.parse_trait_decl(Vis::Private).map(Item::Trait),
            // M-659 / RFC-0019 §4.1: `impl Trait<args> for T { fn … }` (the trait-instance
            // production). `impl` was reserved by M-658 (RFC-0007 §12.2); this is the production that
            // consumes it.
            Tok::Impl => self.parse_impl_decl().map(Item::Impl),
            Tok::Fn | Tok::Thaw => self.parse_fn_decl(Vis::Private).map(Item::Fn),
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
            // M-662: a `phylum` header must be the *first* token of the program (before the nodule
            // blocks); reaching one at item position means it was misplaced after a nodule began.
            Tok::Phylum => Err(ParseError::new(
                self.pos(),
                "a `phylum <path>` header opens the program — it must come before the first \
                 `nodule` block, not at item position (RFC-0006 §4.3; phylum is a grouping, not a \
                 `phylum { … }` container)"
                    .to_owned(),
            )),
            _ => self.err(
                "a top-level item (`use`, `pub`, `default paradigm`, `type`, `trait`, `impl`, `fn`, \
                 or `thaw fn`)",
            ),
        }
    }

    /// Parse a `pub`-prefixed top-level item (M-662). `pub` exports a top-level `fn`/`trait`/`type`
    /// (or `thaw fn`) to the other nodules of the phylum; it is **only** valid there. A `pub use` /
    /// `pub default` / `pub impl` (or `pub` before anything else) is an explicit refusal — never a
    /// silent accept (G2): a `use` imports rather than re-exports, and `impl`/`default` are not part
    /// of the `pub` namespace.
    fn parse_pub_item(&mut self) -> Result<Item, ParseError> {
        self.expect(&Tok::Pub, "`pub`")?;
        match self.cur() {
            Tok::Type => self.parse_type_decl(Vis::Pub).map(Item::Type),
            Tok::Trait => self.parse_trait_decl(Vis::Pub).map(Item::Trait),
            Tok::Fn | Tok::Thaw => self.parse_fn_decl(Vis::Pub).map(Item::Fn),
            Tok::Use => Err(ParseError::new(
                self.pos(),
                "`pub use` is not a form — a `use` imports a name into this nodule, it does not \
                 re-export it (M-662); drop the `pub`"
                    .to_owned(),
            )),
            Tok::Impl => Err(ParseError::new(
                self.pos(),
                "`pub impl` is not a form — an `impl` is not `pub`-gated (its coherence is \
                 phylum-wide and pub-blind; M-662/RFC-0019 §4.5); drop the `pub`"
                    .to_owned(),
            )),
            Tok::Default => Err(ParseError::new(
                self.pos(),
                "`pub default` is not a form — `default paradigm` is nodule-scope ambient state, \
                 not an exportable item (M-662); drop the `pub`"
                    .to_owned(),
            )),
            _ => self.err(
                "`pub` must be followed by `fn`, `trait`, `type`, or `thaw fn` (M-662 — only those \
                 top-level items are exportable)",
            ),
        }
    }

    /// `use path` (specific) or `use path.*` (glob) — a cross-nodule import (M-662; RFC-0006 §4.3).
    /// A trailing `.*` makes it a **glob** (import every `pub` name under the path); otherwise the
    /// path's last segment names the imported item. A `*` anywhere but the final segment is an
    /// explicit parse error — the lexer emits `Tok::Star` for any `*`; this production is what
    /// restricts the glob `*` to the final position. `use` is never `pub`-gated.
    fn parse_use(&mut self) -> Result<UsePath, ParseError> {
        self.expect(&Tok::Use, "`use`")?;
        // A `use` path is a dotted path whose final segment may be `*` (the glob). Parse the dotted
        // path, then check for a trailing `.*`.
        let mut segs = vec![self.ident()?];
        let mut glob = false;
        while self.eat(&Tok::Dot) {
            if self.eat(&Tok::Star) {
                glob = true;
                break;
            }
            segs.push(self.ident()?);
        }
        // A glob needs a prefix to glob under (`use *` alone is meaningless).
        if glob && segs.is_empty() {
            return self.err("a glob `use` needs a path prefix (`use a.b.*`), not a bare `*`");
        }
        Ok(UsePath {
            path: Path(segs),
            glob,
        })
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

    fn parse_type_decl(&mut self, vis: Vis) -> Result<TypeDecl, ParseError> {
        self.expect_keyword(&Tok::Type)?;
        let name = self.ident()?;
        let params = self.parse_type_params_opt()?;
        self.expect(&Tok::Eq, "`=` before the constructors")?;
        let mut ctors = vec![self.parse_ctor()?];
        while self.eat(&Tok::Pipe) {
            ctors.push(self.parse_ctor()?);
        }
        Ok(TypeDecl {
            vis,
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

    fn parse_trait_decl(&mut self, vis: Vis) -> Result<TraitDecl, ParseError> {
        self.expect_keyword(&Tok::Trait)?;
        let name = self.ident()?;
        let params = self.parse_type_params_opt()?;
        self.expect(&Tok::LBrace, "`{` to open the trait body")?;
        let mut sigs = Vec::new();
        while !self.at(&Tok::RBrace) {
            sigs.push(self.parse_fn_sig()?);
        }
        self.expect(&Tok::RBrace, "`}` to close the trait body")?;
        Ok(TraitDecl {
            vis,
            name,
            params,
            sigs,
        })
    }

    fn parse_fn_sig(&mut self) -> Result<FnSig, ParseError> {
        self.expect(&Tok::Fn, "`fn` in the trait body")?;
        self.parse_sig_tail()
    }

    /// `thaw? fn …` with the caller-supplied cross-nodule visibility `vis` (M-662). Top-level fns get
    /// `Vis::Pub` iff a `pub` preceded them; impl methods are always parsed with `Vis::Private` (an
    /// `impl` is not `pub`-gated — its method `vis` is inert).
    fn parse_fn_decl(&mut self, vis: Vis) -> Result<FnDecl, ParseError> {
        let thaw = self.eat(&Tok::Thaw);
        self.expect(&Tok::Fn, "`fn`")?;
        let sig = self.parse_sig_tail()?;
        self.expect(&Tok::Eq, "`=` before the function body")?;
        let body = self.parse_expr()?;
        Ok(FnDecl {
            vis,
            thaw,
            sig,
            body,
        })
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
        let set_start = self.pos();
        let effects = if self.at(&Tok::RBrace) {
            Vec::new()
        } else {
            self.comma_separated(None, Self::ident)?
        };
        self.expect(&Tok::RBrace, "`}` to close the effect set")?;
        if let Some(dup) = first_duplicate_str(&effects) {
            // Point at the effect set itself (not after the closing `}`) for a clearer diagnostic.
            return Err(ParseError::new(
                set_start,
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
            // An impl method is never `pub`-gated (M-662): a `pub` here is an explicit refusal, never
            // a silent accept (G2). Methods are parsed with `Vis::Private` (the field is inert).
            if self.at(&Tok::Pub) {
                return Err(ParseError::new(
                    self.pos(),
                    "an `impl` method is not `pub`-gated — visibility of a trait method follows the \
                     trait, not the impl (M-662); drop the `pub`"
                        .to_owned(),
                ));
            }
            methods.push(self.parse_fn_decl(Vis::Private)?);
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
        // Parse `base [@guarantee]` first.  Then, if a `->` follows, this whole LHS
        // becomes the argument of a function type and we parse the RHS recursively
        // (right-associative; `@` binds tighter than `->` — RFC-0024 §3).
        let lhs = self.parse_type_ref_atom()?;
        if self.eat(&Tok::Arrow) {
            // Right-associative: recurse for the result type (which may itself be `A -> B`).
            let rhs = self.parse_type_ref()?;
            Ok(TypeRef::unguaranteed(BaseType::Fn(
                Box::new(lhs),
                Box::new(rhs),
            )))
        } else {
            Ok(lhs)
        }
    }

    /// Parse a single `base [@guarantee]` atom — without consuming a trailing `->`.  This is
    /// the non-recursive inner step of [`parse_type_ref`]; callers that need to stop *before*
    /// the arrow use this directly (none in v1 — `parse_sig_tail` already consumed its own
    /// `->` before calling `parse_type_ref` for the return type, so there is no ambiguity).
    fn parse_type_ref_atom(&mut self) -> Result<TypeRef, ParseError> {
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
            // RFC-0025 / M-705: the infix-operator layer. A non-keyword expression is an operator
            // expression over unary/applicative operands; each operator desugars to its canonical
            // word function. The keyword-led forms above (let/if/match/…) are statements, not
            // operands; to use one as an operand, parenthesize it.
            _ => self.parse_binexpr(0),
        }
    }

    /// Precedence-climbing parser for infix operator expressions (RFC-0025 / M-705). Each binary
    /// operator desugars to its canonical word function (`a + b` → `add(a, b)`); the word form
    /// remains valid everywhere the sugar is (the sugar is *additive* — words stay canonical, the
    /// kernel is unchanged: this is a frontend-only desugaring, no L0/L1 change, KC-3). `min_bp`
    /// is the minimum binding power this call consumes; left-associativity is encoded by recursing
    /// on the right operand at `bp + 1` so an equal-precedence operator is left for the loop.
    ///
    /// **Stack-safety (A4-02).** This needs no extra depth charge of its own and must not add one
    /// (the enclosing [`parse_expr`](Self::parse_expr) already charges the budget for this
    /// expression level — charging again would double-count and halve the effective nesting limit).
    /// The RHS recursion `parse_binexpr(bp + 1)` strictly *raises* `min_bp` each step, so its
    /// recursion depth is bounded by the fixed number of precedence tiers (a small constant),
    /// independent of input length — it cannot overflow. A flat left-associative chain
    /// `a + a + …` is consumed by the loop (not recursion), so it too stays O(1) deep. The only
    /// genuinely unbounded operator vector is the prefix chain in [`parse_unary`](Self::parse_unary),
    /// which carries its own depth guard. Nested parens route back through `parse_expr` (guarded).
    fn parse_binexpr(&mut self, min_bp: u8) -> Result<Expr, ParseError> {
        let mut lhs = self.parse_unary()?;
        while let Some((bp, word)) = infix_op(self.cur()) {
            if bp < min_bp {
                break;
            }
            self.bump(); // the operator token
            let rhs = self.parse_binexpr(bp + 1)?;
            lhs = op_call(word, vec![lhs, rhs]);
        }
        Ok(lhs)
    }

    /// Prefix unary operators (RFC-0025 / M-705): `-a` → `neg(a)`, `!a` → `not(a)`. Unary binds
    /// tighter than every binary operator and is right-associative (`- - a` → `neg(neg(a))`). A
    /// `!` here is always the unary operator: the effect-set `!{…}` only ever appears in a fn
    /// signature (parsed elsewhere), never at expression position. Any other token delegates to
    /// the applicative layer ([`parse_app`]).
    ///
    /// The prefix recursion participates in the shared [`MAX_EXPR_DEPTH`] budget (A4-02): a crafted
    /// prefix chain (`!!!!…a`, `----a`) is refused with an explicit error past the limit, never a
    /// host-stack overflow (G2 — never crash).
    fn parse_unary(&mut self) -> Result<Expr, ParseError> {
        let word = match self.cur() {
            Tok::Minus => "neg",
            Tok::Bang => "not",
            _ => return self.parse_app(),
        };
        self.depth += 1;
        if self.depth > MAX_EXPR_DEPTH {
            self.depth -= 1;
            return Err(ParseError::new(
                self.pos(),
                format!("expression nests deeper than the limit of {MAX_EXPR_DEPTH} — refusing to recurse"),
            ));
        }
        self.bump(); // the prefix operator
        let operand = self.parse_unary();
        self.depth -= 1;
        operand.map(|o| op_call(word, vec![o]))
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

/// The infix binding power and canonical word function for an operator token (RFC-0025 / M-705),
/// or `None` if the token does not open an infix operator. Higher binding power binds tighter;
/// every binary operator is left-associative. The precedence tiers follow **Rust's** table (the
/// implementation language, syntactically adjacent; RFC-0025 §4), omitting the angle-bracket
/// operators (`<`, `<=`, `>`, `>=`, `<<`, `>>`) which are **deferred** (RFC-0025 §3 / M-745)
/// because `<`/`>` collide with the type-argument `<…>` grammar (resolving that cleanly needs
/// contextual lexing, a separate effort). The desugaring is purely syntactic: a word target whose
/// prim/stdlib function does not yet exist (`div`, `rem`, `band`, `bor`, `eq`, `ne`, `and`, `or`)
/// still desugars here and surfaces an explicit "unknown function/prim" refusal downstream (never
/// silent — G2); only `add`/`sub`/`mul`/`xor` (and unary `neg`/`not`) resolve end-to-end today.
fn infix_op(tok: &Tok) -> Option<(u8, &'static str)> {
    Some(match tok {
        Tok::Star => (70, "mul"),
        Tok::Slash => (70, "div"),
        Tok::Percent => (70, "rem"),
        Tok::Plus => (60, "add"),
        Tok::Minus => (60, "sub"),
        Tok::Amp => (50, "band"),
        Tok::Caret => (40, "xor"),
        Tok::Pipe => (30, "bor"),
        Tok::EqEq => (20, "eq"),
        Tok::BangEq => (20, "ne"),
        Tok::AmpAmp => (11, "and"),
        Tok::PipePipe => (10, "or"),
        _ => return None,
    })
}

/// Build the canonical word-function application an operator desugars to (RFC-0025 / M-705). The
/// sugar leaves **no separate trace**: the desugared `App` node *is* the audit record — the
/// canonical word form is the inspectable EXPLAIN (ADR-006, no black boxes), so `a + b` and
/// `add(a, b)` are structurally identical after parsing (this resolves RFC-0025 Q5 — no separate
/// `DesugarRecord` is needed; the desugaring target is the record).
fn op_call(word: &str, args: Vec<Expr>) -> Expr {
    Expr::App {
        head: Box::new(Expr::Path(Path(vec![word.to_owned()]))),
        args,
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
    use crate::ast::{BaseType, Expr, Item, Literal, TypeRef};

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

    // --- RFC-0025 / M-705: operator syntax (infix sugar desugaring to word functions) ----------

    /// The body of `fn main() -> T = <expr>` for an operator-sugar fixture.
    fn op_body(expr: &str) -> Expr {
        fn_body(&format!("nodule d\nfn main() -> Binary{{8}} = {expr}"))
    }

    #[test]
    fn infix_sugar_desugars_to_the_word_call() {
        // `a + b` is *structurally identical* to `add(a, b)` after parsing — the sugar leaves no
        // separate trace (RFC-0025 Q5: the desugared App node is the EXPLAIN record).
        assert_eq!(op_body("a + b"), op_body("add(a, b)"));
        assert_eq!(op_body("a - b"), op_body("sub(a, b)"));
        assert_eq!(op_body("a * b"), op_body("mul(a, b)"));
        assert_eq!(op_body("a ^ b"), op_body("xor(a, b)"));
        assert_eq!(op_body("a / b"), op_body("div(a, b)"));
        assert_eq!(op_body("a % b"), op_body("rem(a, b)"));
        assert_eq!(op_body("a & b"), op_body("band(a, b)"));
        assert_eq!(op_body("a | b"), op_body("bor(a, b)"));
        assert_eq!(op_body("a == b"), op_body("eq(a, b)"));
        assert_eq!(op_body("a != b"), op_body("ne(a, b)"));
        assert_eq!(op_body("a && b"), op_body("and(a, b)"));
        assert_eq!(op_body("a || b"), op_body("or(a, b)"));
    }

    #[test]
    fn prefix_sugar_desugars_to_the_word_call() {
        assert_eq!(op_body("-a"), op_body("neg(a)"));
        assert_eq!(op_body("!a"), op_body("not(a)"));
        // Prefix is right-associative and binds tighter than any binary op.
        assert_eq!(op_body("- -a"), op_body("neg(neg(a))"));
        assert_eq!(op_body("-a + b"), op_body("add(neg(a), b)"));
    }

    #[test]
    fn precedence_follows_the_rust_table() {
        // `*` (70) > `+` (60): `a + b * c` ≡ `add(a, mul(b, c))`.
        assert_eq!(op_body("a + b * c"), op_body("add(a, mul(b, c))"));
        // `&` (50) > `^` (40) > `|` (30): `a | b ^ c & d` ≡ `bor(a, xor(b, band(c, d)))`.
        assert_eq!(
            op_body("a | b ^ c & d"),
            op_body("bor(a, xor(b, band(c, d)))")
        );
        // arithmetic (60) > equality (20) > `&&` (11) > `||` (10).
        assert_eq!(
            op_body("a + b == c && d || e"),
            op_body("or(and(eq(add(a, b), c), d), e)")
        );
    }

    #[test]
    fn binary_operators_are_left_associative() {
        // `a - b - c` ≡ `sub(sub(a, b), c)`, NOT `sub(a, sub(b, c))`.
        assert_eq!(op_body("a - b - c"), op_body("sub(sub(a, b), c)"));
        assert_eq!(op_body("a + b + c"), op_body("add(add(a, b), c)"));
    }

    #[test]
    fn parens_override_precedence() {
        assert_eq!(op_body("(a + b) * c"), op_body("mul(add(a, b), c)"));
    }

    #[test]
    fn deep_operator_nesting_is_refused_not_crashed() {
        // A4-02 / G2: a crafted prefix chain (`!!!…a`) or parenthesized operator nesting must be
        // refused with an explicit depth error, never drive a host-stack overflow. Both the prefix
        // recursion (parse_unary) and the precedence recursion (parse_binexpr) participate in the
        // shared MAX_EXPR_DEPTH budget.
        let prefix = "!".repeat(2000);
        let src = format!("nodule d\nfn main() -> Binary{{8}} = {prefix}0b0000_0000");
        let err = parse(&src).expect_err("a 2000-deep prefix chain must be refused");
        assert!(
            err.message.contains("refusing to recurse"),
            "got: {}",
            err.message
        );
        // A flat (non-nested) left-associative chain of the SAME length must still parse — the loop
        // keeps it O(1) deep, so length alone never trips the budget.
        let flat = (0..2000)
            .map(|_| "0b0000_0000")
            .collect::<Vec<_>>()
            .join(" ^ ");
        let ok = format!("nodule d\nfn main() -> Binary{{8}} = {flat}");
        parse(&ok).expect("a long FLAT operator chain parses (loop, not recursion)");
    }

    #[test]
    fn word_form_remains_valid_alongside_sugar() {
        // The sugar is additive: the canonical word call still parses (and is a legal operand of
        // sugar). `add(a, b) * c` ≡ `mul(add(a, b), c)`.
        assert_eq!(op_body("add(a, b) * c"), op_body("mul(add(a, b), c)"));
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

    // --- M-660 / RFC-0014 §3.4: effect annotations `!{ … }` on fn signatures parse ---

    #[test]
    fn an_effect_annotation_parses_into_the_signature_effect_set() {
        // `!{io, time}` after the return type lands as the signature's effect set, in source order.
        let n = parse("nodule d\nfn a() -> Binary{8} !{io, time} = 0b00000000").expect("parses");
        let Item::Fn(f) = &n.items[0] else {
            panic!("fn")
        };
        assert_eq!(f.sig.effects, vec!["io".to_owned(), "time".to_owned()]);
    }

    #[test]
    fn an_unannotated_fn_has_an_empty_effect_set_and_an_explicit_empty_set_too() {
        // Unannotated ⇒ pure (empty set); the explicit written `!{}` is also the empty set — both
        // mean "declares no effects" (RFC-0014 I5).
        let plain = parse("nodule d\nfn a() -> Binary{8} = 0b00000000").expect("parses");
        let Item::Fn(f) = &plain.items[0] else {
            panic!("fn")
        };
        assert!(f.sig.effects.is_empty());
        let empty = parse("nodule d\nfn a() -> Binary{8} !{} = 0b00000000").expect("parses");
        let Item::Fn(f) = &empty.items[0] else {
            panic!("fn")
        };
        assert!(f.sig.effects.is_empty());
    }

    #[test]
    fn a_trait_method_requirement_carries_an_effect_annotation() {
        // The effect annotation is part of the shared signature tail, so a trait method requirement
        // (no body) carries it too (the impl-vs-trait effect conformance check consumes it — M-660).
        let n = parse("nodule d\ntrait T<A> { fn m(x: A) -> A !{io} }").expect("parses");
        let Item::Trait(td) = &n.items[0] else {
            panic!("trait")
        };
        assert_eq!(td.sigs[0].effects, vec!["io".to_owned()]);
    }

    #[test]
    fn a_bare_bang_without_an_effect_brace_is_an_explicit_error() {
        // `!` only ever opens an effect set; a `!` not followed by `{` is a never-silent parse error
        // (v0 has no negation/`not` operator — logical ops are named prims; G2).
        let err = parse("nodule d\nfn a() -> Binary{8} ! = 0b00000000")
            .expect_err("a bare `!` must be rejected");
        assert!(err.message.contains("effect set"), "got: {}", err.message);
    }

    // --- M-661 / RFC-0016 §8-Q6: the `@std-sys` nodule-header FFI-floor marker parses ---

    #[test]
    fn the_std_sys_header_marker_sets_the_nodule_flag() {
        // `nodule <path> @std-sys` sets `Nodule.std_sys`; a plain `nodule <path>` leaves it false.
        // The marker is an attribute on the header, parsed after the path (M-661).
        let marked =
            parse("nodule std.sys.fs @std-sys\nfn f() -> Binary{1} = 0b0").expect("parses");
        assert!(marked.std_sys, "the @std-sys marker must set std_sys");
        assert_eq!(marked.path.0, vec!["std", "sys", "fs"]);
        let plain = parse("nodule d\nfn f() -> Binary{1} = 0b0").expect("parses");
        assert!(!plain.std_sys, "an unmarked nodule is not std-sys");
    }

    #[test]
    fn a_std_sys_nodule_parses_a_wild_block_in_a_fn_body() {
        // The marker + a `wild` block parse together (the context gate + effect coverage are CHECKER
        // concerns, not parse concerns — this only pins that the surface admits both).
        let n =
            parse("nodule std.sys.x @std-sys\nfn f() -> Binary{8} !{ffi} = wild { host_call() }")
                .expect("a @std-sys nodule with a wild block parses");
        assert!(n.std_sys);
        let Item::Fn(fd) = &n.items[0] else {
            panic!("fn")
        };
        assert!(matches!(fd.body, Expr::Wild(_)), "the body is a wild block");
        assert_eq!(fd.sig.effects, vec!["ffi".to_owned()]);
    }

    // --- M-685 / RFC-0024 §3: function type `A -> B` surface + fn-name-as-value ---

    /// Helper: extract the `TypeRef` of the first named parameter of the first `fn` item.
    fn first_param_ty(src: &str) -> TypeRef {
        let n = parse(src).expect("parses");
        n.items
            .into_iter()
            .find_map(|i| match i {
                Item::Fn(fd) => Some(fd.sig.value_params.into_iter().next()?.ty),
                _ => None,
            })
            .expect("a fn with at least one value parameter")
    }

    #[test]
    fn simple_fn_type_parses_to_basetype_fn() {
        // `f: A -> B` in a parameter builds `BaseType::Fn(Named("A"), Named("B"))`.
        // Use a single-param fn so `first_param_ty` finds the fn-typed one directly.
        let ty = first_param_ty("nodule d\nfn apply<A, B>(f: A -> B) -> B = f");
        let BaseType::Fn(arg, ret) = ty.base else {
            panic!("expected BaseType::Fn, got {:?}", ty.base);
        };
        assert!(
            matches!(arg.base, BaseType::Named(ref n, _) if n == "A"),
            "arg should be Named(A), got {:?}",
            arg.base
        );
        assert!(
            matches!(ret.base, BaseType::Named(ref n, _) if n == "B"),
            "ret should be Named(B), got {:?}",
            ret.base
        );
        assert!(ty.guarantee.is_none(), "no guarantee on the outer fn type");
    }

    #[test]
    fn fn_type_is_right_associative() {
        // `A -> B -> C` must parse as `A -> (B -> C)`.
        let ty = first_param_ty("nodule d\nfn f<A, B, C>(g: A -> B -> C) -> A = g");
        // Outer is `Fn(A, B -> C)`.
        let BaseType::Fn(arg, ret) = ty.base else {
            panic!("expected outer BaseType::Fn");
        };
        assert!(matches!(arg.base, BaseType::Named(ref n, _) if n == "A"));
        // Inner `ret` must itself be `Fn(B, C)`.
        let BaseType::Fn(b, c) = ret.base else {
            panic!(
                "expected inner BaseType::Fn (right-assoc), got {:?}",
                ret.base
            );
        };
        assert!(matches!(b.base, BaseType::Named(ref n, _) if n == "B"));
        assert!(matches!(c.base, BaseType::Named(ref n, _) if n == "C"));
    }

    #[test]
    fn guarantee_binds_tighter_than_arrow() {
        // `A @ Exact -> B` must parse as `(A @ Exact) -> B`.
        let ty = first_param_ty("nodule d\nfn f<A, B>(g: A @ Exact -> B) -> B = g");
        let BaseType::Fn(arg, _ret) = ty.base else {
            panic!("expected BaseType::Fn");
        };
        // The LHS `(A @ Exact)` carries the Exact guarantee; the outer fn type has none.
        assert!(
            matches!(arg.guarantee, Some(crate::ast::Strength::Exact)),
            "arg should carry Exact guarantee, got {:?}",
            arg.guarantee
        );
        assert!(ty.guarantee.is_none(), "outer fn type has no guarantee");
    }

    #[test]
    fn rfc_0024_map_snippet_parses() {
        // RFC-0024 §3's canonical snippet: `fn map<A, B, E>(r: Result<A,E>, f: A -> B) -> Result<B,E>`.
        // Structural check: two value params, second has type `BaseType::Fn`.
        let n = parse(
            "nodule d\n\
             type Result<A, E> = Ok(A) | Err(E)\n\
             fn map<A, B, E>(r: Result<A, E>, f: A -> B) -> Result<B, E> =\
               match r { Ok(x) => Ok(f(x)), Err(e) => Err(e) }",
        )
        .expect("RFC-0024 §3 map snippet parses");
        let Item::Fn(fd) = n
            .items
            .iter()
            .find(|i| matches!(i, Item::Fn(_)))
            .expect("fn")
        else {
            panic!("fn");
        };
        assert_eq!(fd.sig.name, "map");
        assert_eq!(fd.sig.value_params.len(), 2);
        let f_ty = &fd.sig.value_params[1].ty;
        assert!(
            matches!(f_ty.base, BaseType::Fn(_, _)),
            "second param `f` should have a function type, got {:?}",
            f_ty.base
        );
    }

    #[test]
    fn bare_fn_name_in_value_position_parses_as_path() {
        // `map(mk_ok(), double)` — `double` in value (non-call) position is `Expr::Path`, not
        // `Expr::App`.  This confirms fn-as-value needs no parser change (RFC-0024 §3).
        let n = parse(
            "nodule d\n\
             type Result<A, E> = Ok(A) | Err(E)\n\
             fn double<A>(x: A) -> A = x\n\
             fn mk_ok<A>(x: A) -> Result<A, A> = Ok(x)\n\
             fn map<A, B, E>(r: Result<A, E>, f: A -> B) -> Result<B, E> =\
               match r { Ok(x) => Ok(f(x)), Err(e) => Err(e) }\n\
             fn main() -> Result<Binary{8}, Binary{8}> = map(mk_ok(0b00000000), double)",
        )
        .expect("parses");
        // Find the `main` fn and inspect its body.
        let Item::Fn(main_fd) = n
            .items
            .iter()
            .find(|i| matches!(i, Item::Fn(fd) if fd.sig.name == "main"))
            .expect("main fn")
        else {
            panic!("main fn");
        };
        // Body is `map(mk_ok(0b00000000), double)` → `App { head: Path([map]), args: [App(mk_ok), Path([double])] }`.
        let Expr::App { ref head, ref args } = main_fd.body else {
            panic!("expected App, got {:?}", main_fd.body);
        };
        assert!(matches!(head.as_ref(), Expr::Path(p) if p.0 == vec!["map"]));
        assert_eq!(args.len(), 2);
        // First arg: `mk_ok(0b00000000)` — an App.
        assert!(
            matches!(args[0], Expr::App { .. }),
            "first arg is App (call)"
        );
        // Second arg: `double` — a bare Path (fn-as-value, no call parens).
        assert!(
            matches!(args[1], Expr::Path(ref p) if p.0 == vec!["double"]),
            "second arg `double` should be a bare Path, got {:?}",
            args[1]
        );
    }

    #[test]
    fn malformed_arrow_missing_rhs_is_explicit_error() {
        // `A ->` with no right-hand type must be an explicit `ParseError` — never silently accepted
        // (G2 / house rule #2: never-silent).
        let err = parse("nodule d\nfn f<A>(g: A ->) -> A = g")
            .expect_err("a bare `A ->` with no rhs must be rejected");
        // The error should describe what was missing — a type is expected after `->`.
        assert!(
            err.message.contains("type") || err.message.contains("expected"),
            "error message should mention a missing type: {:?}",
            err.message
        );
    }

    #[test]
    fn fn_type_in_return_position_parses() {
        // A function may return a function type: `fn make_fn<A, B>() -> A -> B = ...`
        // The `->` in the return type is also right-associative and fully parsed.
        let n = parse("nodule d\nfn make_fn<A, B>(x: A) -> A -> B = x").expect("parses");
        let Item::Fn(fd) = &n.items[0] else {
            panic!("fn")
        };
        assert!(
            matches!(fd.sig.ret.base, BaseType::Fn(_, _)),
            "return type should be BaseType::Fn, got {:?}",
            fd.sig.ret.base
        );
    }
}
