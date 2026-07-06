//! The `.myc` emitter (M-873).
//!
//! Every emission path here is a `match` over a `syn` node, and every fallback/uncovered arm
//! returns `Err(GapReason)` rather than emitting a placeholder or dropping the construct — the
//! driver (`transpile.rs`) is responsible for turning every `Err` into a recorded [`Gap`] (never
//! silent, G2). Nothing in this module ever writes a partial or best-guess `.myc` fragment for a
//! construct it isn't confident about; "confident" here means "traced to a specific grammar
//! production in `docs/spec/grammar/mycelium.ebnf`", cited in the comments below.
//!
//! **Guarantee: `Declared`.** All emitted text is heuristic, unvalidated by any Mycelium
//! parser/typechecker (see crate docs).

use crate::gap::{guarded, Category, GapReason};
use crate::map::{map_type, tokens_to_string};
use crate::reserved::guard_ident;
use syn::{
    Attribute, Block, Expr, Fields, FnArg, GenericArgument, GenericParam, Generics, ImplItem,
    ItemEnum, ItemFn, ItemImpl, ItemStruct, ItemTrait, Lit, Pat, PathArguments, ReturnType,
    Signature, Stmt, TraitItem,
};

/// The `.myc` text (+ any dropped sub-features, e.g. attributes) for one successfully emitted
/// top-level item.
pub struct Emitted {
    pub name: String,
    pub myc: String,
    /// Sub-features of this *otherwise-emitted* item that were still dropped (e.g. a
    /// `#[derive(..)]`, or — for an `impl` block — a method that individually failed to map).
    /// Recorded so the item can be simultaneously "emitted" (its core structure landed) and
    /// "in gaps" (something about it is honestly flagged) — both is allowed; only "neither" is
    /// forbidden (see `GapReport` docs).
    pub sub_gaps: Vec<GapReason>,
}

// ---------------------------------------------------------------------------------------------
// Shared helpers: doc/attr extraction, generic-parameter mapping, fn-signature mapping.
// ---------------------------------------------------------------------------------------------

/// Extract `///`/`//!` doc-comment lines (represented by `syn` as `#[doc = "..."]` attributes),
/// rendered as plain `//` line comments (grammar: "line comments start with '//' ... ignored by
/// the grammar" — doc comments have no first-class surface form, so this is the closest honest
/// mapping: preserved as prose, not as a structured doc construct).
pub fn doc_lines(attrs: &[Attribute]) -> Vec<String> {
    let mut lines = Vec::new();
    for attr in attrs {
        if attr.path().is_ident("doc") {
            if let syn::Meta::NameValue(nv) = &attr.meta {
                if let Expr::Lit(syn::ExprLit {
                    lit: Lit::Str(s), ..
                }) = &nv.value
                {
                    lines.push(format!("//{}", s.value()));
                }
            }
        }
    }
    lines
}

/// Every non-doc attribute on an item, rendered as text — these are always dropped (KNOWN HARD
/// GAP: derive/`#[...]` attributes have no confirmed Mycelium surface), recorded via a
/// [`Category::DeriveAttr`] sub-gap rather than silently discarded.
pub fn non_doc_attrs(attrs: &[Attribute]) -> Vec<String> {
    attrs
        .iter()
        .filter(|a| !a.path().is_ident("doc"))
        .map(tokens_to_string)
        .collect()
}

/// Heuristic `#[cfg(test)]` detection (Declared: a token-text `contains("test")` check, not a
/// real `cfg` predicate evaluator).
pub fn is_cfg_test(attrs: &[Attribute]) -> bool {
    attrs
        .iter()
        .any(|a| a.path().is_ident("cfg") && tokens_to_string(a).contains("test"))
}

/// Map a `Generics` list to Mycelium's bare `type_params ::= '[' Ident (',' Ident)* ']'` —
/// confirmed to allow *only* unbounded type identifiers (grammar comment: "a fn generic over
/// both is `[T]{N}`"; bounds live on individual `fn` params via `RFC-0019 §4.1`, not on the
/// type-param list itself in this fragment). A lifetime, a bounded type param, or a const
/// generic each has no confirmed slot here.
fn plain_type_params(generics: &Generics) -> Result<Vec<String>, GapReason> {
    if generics.where_clause.is_some() {
        return Err(GapReason::new(
            Category::WhereClause,
            "a `where` clause has no Mycelium equivalent",
        ));
    }
    let mut names = Vec::new();
    for p in &generics.params {
        match p {
            GenericParam::Type(tp) => {
                if !tp.bounds.is_empty() {
                    return Err(GapReason::new(
                        Category::GenericBound,
                        format!(
                            "type parameter `{}` carries a bound — type_params/fn generics are \
                             bare identifiers only in this grammar fragment",
                            tp.ident
                        ),
                    ));
                }
                // Same emit-verbatim exposure as fn parameters: an UNUSED type-param name never
                // reaches map_type's guard, so guard at the declaration site too.
                crate::reserved::guard_ident(&tp.ident.to_string(), "type parameter")?;
                names.push(tp.ident.to_string());
            }
            GenericParam::Lifetime(lt) => {
                return Err(GapReason::new(
                    Category::GenericBound,
                    format!(
                        "lifetime parameter `{}` has no grammar surface",
                        lt.lifetime
                    ),
                ));
            }
            GenericParam::Const(cp) => {
                return Err(GapReason::new(
                    Category::GenericBound,
                    format!(
                        "const generic parameter `{}` — correspondence with Mycelium's width \
                         const_params (`{{N}}`) is not confirmed",
                        cp.ident
                    ),
                ));
            }
        }
    }
    Ok(names)
}

// ---------------------------------------------------------------------------------------------
// DN-41 `width_cast` conversion-body emission (M-873 follow-on).
//
// `docs/notes/DN-41-Width-Cast-Prim.md` §2 ratifies a real surface prim
// `width_cast(value: Binary{N}, into: Binary{M}) -> Binary{M}`: widen (M>N) zero-extends
// (`Exact`); same-width is identity; narrow (M<N) is a checked, never-silent refuse
// (`EvalError::Overflow`) — §3 fixes the **width-witness ABI**: `M` is carried by the *second
// operand's* `Binary{M}` width alone (its bits are unused), exactly as `lib/std/text.myc`'s own
// `width_cast(i, bytes_len(b))` call threads a width through an in-scope `Binary{32}` value.
//
// A Rust `impl Widen<To> for From { fn widen(self) -> To { To::from(self) } }` body — the actual
// shape in `mycelium-std-cmp` — has no confirmed mapping for the qualified `To::from(self)` call
// (see `emit_expr`'s `Expr::Call` qualified-path arm); previously that always gapped the whole
// impl. When `From`/`To` both map to `Binary{N}`/`Binary{M}` (unsigned widening), this is now a
// **real, faithful** emission instead: `width_cast(self, <Binary{M} witness>)`. The witness is a
// synthesized all-zero `BinLit` of exactly `M` bits — confirmed as a legitimate `Binary{M}`-typed
// value by the grammar (`literal ::= BinLit | ...`, `BinLit ::= '0b' ('0'|'1'|'_')+`) and
// RFC-0020 §"Representation-tagged literals" ("[a BinLit's] width/dimension is determined by the
// literal's content (bit-count for BinLit)") — and DN-41 §3 explicitly says the witness's *bits*
// are ignored, so an all-zero witness is exactly as valid as any other same-width value already
// in scope. This is a synthesized witness, not one reused from the call site (the widen body has
// no other `Binary{M}` value in scope to reuse) — `Declared`, not `Exact`, because no Mycelium
// checker in this crate confirms the emitted text type-checks (see module docs).
//
// `Narrow::narrow` bodies are the DN-41 §2 fallible case (`Result<To, NarrowError>`, refusing on
// an out-of-range/non-representable value) — a single `= expr` `fn_item` body has no
// Result-returning surface in this grammar fragment, so those stay an honest, explicitly-cited
// gap rather than a forced/fabricated emission.

/// Parse a `map_type`-produced `Binary{N}` type-ref string back to its width `N`. Only matches
/// the exact `Binary{<digits>}` shape `map_type` emits for unsigned integers — never a guess for
/// any other text (e.g. `Bool`, a bare ident) that happens to not match.
fn binary_width(ty_text: &str) -> Option<u32> {
    ty_text
        .strip_prefix("Binary{")
        .and_then(|rest| rest.strip_suffix('}'))
        .and_then(|digits| digits.parse::<u32>().ok())
}

/// Synthesize an all-zero `BinLit` witness of exactly `width` bits, grouped in nibbles
/// (`0b0000_0000_0000_0000` for width 16) matching the corpus's own `BinLit` style (e.g.
/// `lib/std/text.myc`'s `0b0000_0000_0000_0000_0000_0000_1000_0000`). The witness's bits are
/// ignored by `width_cast` (DN-41 §3) — only its bit-count (= its `Binary{width}` type, per
/// RFC-0020) is observed, so an all-zero pattern is a faithful, unconditionally-valid witness for
/// any target width.
fn zero_bin_literal(width: u32) -> String {
    let mut s = String::with_capacity(2 + width as usize + width as usize / 4);
    s.push_str("0b");
    for i in 0..width {
        if i > 0 && i % 4 == 0 {
            s.push('_');
        }
        s.push('0');
    }
    s
}

/// If `trait_name`/`method` identify a `Widen::widen` method whose `Self`/target both map to
/// `Binary{N}`/`Binary{M}` (unsigned widening) with `M > N`, return the faithful `width_cast`
/// body. `None` for every other shape (bool/float/signed self types, non-`Widen` impls, or a
/// `Widen` impl whose recorded target arg isn't a plain `Binary{M}` text) — the caller falls back
/// to the general per-expression emitter, which gaps `To::from(self)` honestly (no fabrication,
/// VR-5).
fn try_width_cast_widen_body(
    trait_name: Option<&str>,
    method: &str,
    self_ty_text: &str,
    trait_targs: &[String],
) -> Option<String> {
    if trait_name != Some("Widen") || method != "widen" {
        return None;
    }
    let n = binary_width(self_ty_text)?;
    let m = binary_width(trait_targs.first()?)?;
    if m <= n {
        // Not an actual widen (or an unresolvable width relationship) — leave it to the general
        // path rather than emit a `width_cast` that DN-41 would treat as identity/narrow for a
        // trait that promises "Total — never fails" widening. Never guessed (VR-5).
        return None;
    }
    Some(format!("width_cast(self, {})", zero_bin_literal(m)))
}

/// Reject `async`/`unsafe`/`extern "ABI"` fn modifiers — `fn_item`/`fn_sig` in the grammar carry
/// no such modifier slot.
fn check_fn_modifiers(sig: &Signature) -> Result<(), GapReason> {
    if sig.asyncness.is_some() || sig.unsafety.is_some() || sig.abi.is_some() {
        return Err(GapReason::new(
            Category::Other,
            "`async`/`unsafe`/`extern \"ABI\"` fn modifier has no grammar surface",
        ));
    }
    Ok(())
}

struct MappedSig {
    params: Vec<(String, String)>,
    ret: String,
    type_params: Vec<String>,
}

/// Map a fn signature's generics/params/return type. `self_ty` is `Some(name)` inside an
/// impl/trait body (the concrete or best-effort `Self` substitution); `None` for a top-level fn,
/// where a `self` parameter or bare `Self` type is therefore always a gap.
fn map_signature(
    generics: &Generics,
    inputs: &syn::punctuated::Punctuated<FnArg, syn::token::Comma>,
    output: &ReturnType,
    self_ty: Option<&str>,
) -> Result<MappedSig, GapReason> {
    let type_params = plain_type_params(generics)?;
    let mut params = Vec::with_capacity(inputs.len());
    for arg in inputs {
        match arg {
            FnArg::Receiver(r) => {
                if r.reference.is_some() && r.mutability.is_some() {
                    return Err(GapReason::new(
                        Category::Other,
                        "`&mut self` conflicts with Mycelium's value semantics (ADR-003) — no \
                         correspondence",
                    ));
                }
                let ty = self_ty.ok_or_else(|| {
                    GapReason::new(
                        Category::Other,
                        "`self` parameter with no enclosing impl/trait context",
                    )
                })?;
                params.push(("self".to_string(), ty.to_string()));
            }
            FnArg::Typed(pt) => {
                let name = match &*pt.pat {
                    Pat::Ident(pi) if pi.by_ref.is_none() && pi.subpat.is_none() => {
                        pi.ident.to_string()
                    }
                    _ => {
                        return Err(GapReason::new(
                            Category::Other,
                            "non-identifier parameter pattern (destructuring param) has no \
                             `param ::= Ident ':' type_ref` equivalent",
                        ))
                    }
                };
                // A parameter name is emitted verbatim into `param ::= Ident ':' type_ref`, and
                // an UNUSED param's body references never pass through Expr::Path — so the
                // reserved-word guard must fire here, not only at use sites (PR #1207 review).
                crate::reserved::guard_ident(&name, "fn parameter")?;
                let ty = map_type(&pt.ty, self_ty)?;
                params.push((name, ty));
            }
        }
    }
    let ret = match output {
        ReturnType::Default => {
            return Err(GapReason::new(
                Category::Other,
                "function has no return type (implicit `()`) — no unit value is representable \
                 in this grammar fragment",
            ))
        }
        ReturnType::Type(_, ty) => map_type(ty, self_ty)?,
    };
    Ok(MappedSig {
        params,
        ret,
        type_params,
    })
}

fn render_fn(name: &str, sig: &MappedSig, body: &str, doc: &[String]) -> String {
    let params_str = sig
        .params
        .iter()
        .map(|(n, t)| format!("{n}: {t}"))
        .collect::<Vec<_>>()
        .join(", ");
    let type_params_text = if sig.type_params.is_empty() {
        String::new()
    } else {
        format!("[{}]", sig.type_params.join(", "))
    };
    let mut out = String::new();
    for d in doc {
        out.push_str(d);
        out.push('\n');
    }
    out.push_str(&format!(
        "fn {name}{type_params_text}({params_str}) => {} = {body};",
        sig.ret
    ));
    out
}

fn render_fn_sig(name: &str, sig: &MappedSig) -> String {
    let params_str = sig
        .params
        .iter()
        .map(|(n, t)| format!("{n}: {t}"))
        .collect::<Vec<_>>()
        .join(", ");
    let type_params_text = if sig.type_params.is_empty() {
        String::new()
    } else {
        format!("[{}]", sig.type_params.join(", "))
    };
    format!("fn {name}{type_params_text}({params_str}) => {}", sig.ret)
}

// ---------------------------------------------------------------------------------------------
// Function bodies: a `let`-chain + tail expression maps to Mycelium's nested `let ... in ...`;
// anything else (early return, loops, multiple non-`let` statements, no tail expr) is a
// MultiStmtBody gap — a KNOWN HARD GAP named in the kickoff brief.
// ---------------------------------------------------------------------------------------------

pub fn emit_block_as_expr(block: &Block, self_ty: Option<&str>) -> Result<String, GapReason> {
    guarded(|| emit_block_as_expr_inner(block, self_ty))
}

/// The recursion-guarded body of [`emit_block_as_expr`] (RFC-0041 §4.7 W1 — see
/// `crate::gap::guarded`). Every recursive call back into a guarded entry point uses the *public*
/// wrapper name (`emit_expr`, `emit_block_as_expr` is not itself re-entered here), so each
/// recursion step consumes one budget frame.
fn emit_block_as_expr_inner(block: &Block, self_ty: Option<&str>) -> Result<String, GapReason> {
    let stmts = &block.stmts;
    if stmts.is_empty() {
        return Err(GapReason::new(
            Category::MultiStmtBody,
            "empty function body (no expression)",
        ));
    }
    let (lets, tail) = stmts.split_at(stmts.len() - 1);
    let tail_expr = match &tail[0] {
        Stmt::Expr(e, None) => e,
        _ => {
            return Err(GapReason::new(
                Category::MultiStmtBody,
                "function body's last statement is not a trailing expression (implicit unit \
                 return, or a semicolon-terminated final statement)",
            ))
        }
    };
    let mut bindings = Vec::with_capacity(lets.len());
    for s in lets {
        match s {
            Stmt::Local(local) => {
                let name =
                    match &local.pat {
                        Pat::Ident(pi) if pi.by_ref.is_none() && pi.subpat.is_none() => {
                            pi.ident.to_string()
                        }
                        _ => return Err(GapReason::new(
                            Category::MultiStmtBody,
                            "`let` binding uses an unsupported pattern (only simple `let x = e;` \
                             is supported)",
                        )),
                    };
                let init = local.init.as_ref().ok_or_else(|| {
                    GapReason::new(Category::MultiStmtBody, "`let` binding has no initializer")
                })?;
                if init.diverge.is_some() {
                    return Err(GapReason::new(
                        Category::MultiStmtBody,
                        "`let ... else` has no Mycelium equivalent",
                    ));
                }
                let value = emit_expr(&init.expr, self_ty)?;
                bindings.push((name, value));
            }
            // A non-`let`, non-tail statement — name the actual kind so the gap reason is precise
            // (never-silent, G2). syn's `Stmt` is a plain 4-variant enum (`Local` handled above).
            Stmt::Item(_) => {
                return Err(GapReason::new(
                    Category::MultiStmtBody,
                    "function body contains a nested item declaration (e.g. a local \
                     `static`/`const`/`fn`) — this grammar fragment has no local-item production; \
                     only simple `let x = e;` bindings plus a trailing expression map",
                ))
            }
            Stmt::Macro(_) => {
                return Err(GapReason::new(
                    Category::MultiStmtBody,
                    "function body contains a macro-invocation statement (e.g. \
                     `debug_assert!`/`println!`) — no macro system in this grammar fragment",
                ))
            }
            Stmt::Expr(_, _) => {
                return Err(GapReason::new(
                    Category::MultiStmtBody,
                    "function body has a semicolon-terminated (value-discarding) statement \
                     expression before the tail — a `let`-chain body maps only simple `let x = e;` \
                     bindings plus a single trailing expression",
                ))
            }
        }
    }
    let mut result = emit_expr(tail_expr, self_ty)?;
    for (name, value) in bindings.into_iter().rev() {
        result = format!("let {name} = {value} in {result}");
    }
    Ok(result)
}

/// Re-encode a Rust string value into a Mycelium `StrLit` (grammar `literal ::= … | StrLit`,
/// line 414; `StrLit ::= '"' (StrChar | EscapeSeq)* '"'`, line 430; M-910/M-911). `syn` hands us
/// the *decoded* string value, so re-escape it into Mycelium's deliberately-minimal escape set
/// (`EscapeSeq ::= '\' ('n' | 't' | '\\' | '"' | '0' | 'r')`, line 433). A control character with
/// no Mycelium escape is a never-silent gap, not a raw-byte injection: Mycelium has no `\xNN`/
/// `\u{..}` form (grammar §StrLit note, lines 424-428), so such a char *cannot* be faithfully
/// represented (G2/VR-5). Every other char — including non-ASCII like `μ` — is a valid `StrChar`
/// (`[^"\\\n\r]`, line 431) that lowers to its UTF-8 bytes (line 427), so it is emitted verbatim.
fn myc_string_literal(value: &str) -> Result<String, GapReason> {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for c in value.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\0' => out.push_str("\\0"),
            c if c.is_control() => {
                return Err(GapReason::new(
                    Category::Other,
                    format!(
                        "string literal contains control character U+{:04X} with no Mycelium \
                         escape — StrLit's escape set is exactly `\\n \\t \\\\ \\\" \\0 \\r` (no \
                         `\\xNN`/`\\u{{..}}` form; grammar §StrLit/EscapeSeq, M-910/M-911), so it \
                         cannot be faithfully represented",
                        c as u32
                    ),
                ))
            }
            c => out.push(c),
        }
    }
    out.push('"');
    Ok(out)
}

/// Whether `digits` (a `syn::LitFloat::base10_digits()` string — the suffix already stripped and
/// underscores removed by `syn`) is a well-formed Mycelium `FloatLit` (grammar lines 443-445:
/// `[0-9]+ '.' [0-9]+ Exponent?` or `[0-9]+ Exponent`; `Exponent ::= ('e' | 'E') ('+' | '-')?
/// [0-9]+`). Only an exact shape match returns `true` — a Rust-only form (a bare `1f64` → "1", a
/// trailing-dot `2.` → "2.") returns `false` and is gapped rather than reshaped, so the emitter
/// never synthesizes a literal the source did not already spell (VR-5). (`syn` normalizes `E`→`e`,
/// drops a `+` exponent sign, and strips underscores, all of which stay within this grammar.)
fn is_myc_float_literal(digits: &str) -> bool {
    let (mantissa, exp) = match digits.find(['e', 'E']) {
        Some(i) => (&digits[..i], Some(&digits[i + 1..])),
        None => (digits, None),
    };
    if let Some(e) = exp {
        let e = e.strip_prefix(['+', '-']).unwrap_or(e);
        if e.is_empty() || !e.bytes().all(|b| b.is_ascii_digit()) {
            return false;
        }
    }
    let all_digits = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
    match mantissa.split_once('.') {
        // `[0-9]+ '.' [0-9]+` (Exponent already validated above if present).
        Some((int, frac)) => all_digits(int) && all_digits(frac),
        // `[0-9]+ Exponent` — a dot-less mantissa is a FloatLit *only* with an exponent (else it
        // is an `Int`, not a float — Mycelium's structural Int/float disambiguation, grammar
        // line 437).
        None => exp.is_some() && all_digits(mantissa),
    }
}

/// Translate one Rust expression. Exhaustive `match` over `syn::Expr` (itself `#[non_exhaustive]`
/// — the trailing `_` arm is therefore also the forward-compatibility catch-all); every arm not
/// explicitly handled falls to that final arm, which returns `Err`, never emits a placeholder.
///
/// **RFC-0041 §4.7 (W1):** guarded by the crate-wide recursion budget (`crate::gap::guarded`) —
/// mutually recurses with [`emit_block_as_expr`]/[`map_pattern`] over unbounded/attacker-controlled
/// input depth (e.g. deeply-parenthesized `Expr::Paren`), so each call consumes one budget frame
/// and refuses with a `Category::RecursionBudget` gap rather than risking a host-stack overflow.
pub fn emit_expr(expr: &Expr, self_ty: Option<&str>) -> Result<String, GapReason> {
    guarded(|| emit_expr_inner(expr, self_ty))
}

/// The recursion-guarded body of [`emit_expr`] (see [`emit_expr`]'s docs / `crate::gap::guarded`).
/// Recursive calls within this match use the public `emit_expr` name so each nested call re-enters
/// the guard.
fn emit_expr_inner(expr: &Expr, self_ty: Option<&str>) -> Result<String, GapReason> {
    match expr {
        Expr::Path(p) if p.qself.is_none() => {
            // Declared mapping decision: a qualified path (`Type::Variant`, UFCS calls) is
            // reduced to its last segment — Mycelium constructor/value references are bare
            // identifiers within a nodule (matching `lib/std/cmp.myc`'s own style, e.g. `Lt`
            // rather than `Ordering.Lt`); this transpiler emits everything into one nodule, so
            // qualification carries no distinguishing information here.
            let seg = p
                .path
                .segments
                .last()
                .ok_or_else(|| GapReason::new(Category::Other, "empty path expression"))?;
            let name = seg.ident.to_string();
            guard_ident(&name, "value/constructor reference")?;
            Ok(name)
        }
        Expr::Lit(l) => match &l.lit {
            Lit::Bool(b) => Ok(if b.value { "True" } else { "False" }.to_string()),
            Lit::Int(i) => Ok(i.base10_digits().to_string()),
            // A Rust string literal maps to a Mycelium `StrLit` (grammar `literal ::= … | StrLit`,
            // line 414; M-910/M-911). `myc_string_literal` re-escapes into Mycelium's minimal
            // escape set and gaps (never-silent) on a char it cannot faithfully represent.
            Lit::Str(s) => myc_string_literal(&s.value()),
            // A Rust float literal maps to a Mycelium `FloatLit` (grammar `literal ::= … | FloatLit`,
            // line 414 / `FloatLit`, line 443; ADR-040/M-897) — but *only* when its `syn`-normalized
            // digit string is already a well-formed FloatLit AND denotes a finite binary64 value
            // (ADR-040 §2.4: a literal is a conversion boundary, out-of-range is a never-silent
            // refuse, so a non-finite `1e999` never lands on ±inf). A Rust-only shape or a
            // non-finite value is gapped rather than reshaped/forced (VR-5).
            Lit::Float(f) => {
                let digits = f.base10_digits();
                if !is_myc_float_literal(digits) {
                    Err(GapReason::new(
                        Category::Other,
                        format!(
                            "float literal `{digits}` has no faithful Mycelium `FloatLit` spelling \
                             (FloatLit is `[0-9]+ '.' [0-9]+ Exponent?` | `[0-9]+ Exponent`, no \
                             trailing-dot/bare-suffix form — grammar line 443; ADR-040/M-897)"
                        ),
                    ))
                } else if !f.base10_parse::<f64>().is_ok_and(f64::is_finite) {
                    Err(GapReason::new(
                        Category::Other,
                        format!(
                            "float literal `{digits}` is not a finite binary64 value — a literal \
                             is a conversion boundary, so out-of-range is a never-silent refuse, \
                             never a silent ±inf (ADR-040 §2.4 / FloatLit note, grammar line 439)"
                        ),
                    ))
                } else {
                    Ok(digits.to_string())
                }
            }
            _ => Err(GapReason::new(
                Category::Other,
                format!(
                    "unsupported literal kind `{}` (only bool/int/string/float literals map)",
                    tokens_to_string(l)
                ),
            )),
        },
        Expr::If(e) => {
            let else_branch = e.else_branch.as_ref().ok_or_else(|| {
                GapReason::new(
                    Category::Other,
                    "`if` without an `else` branch — if_expr requires both arms",
                )
            })?;
            if matches!(*e.cond, Expr::Let(_)) {
                return Err(GapReason::new(
                    Category::Other,
                    "`if let` has no Mycelium equivalent in this grammar fragment",
                ));
            }
            let cond = emit_expr(&e.cond, self_ty)?;
            let then_ = emit_block_as_expr(&e.then_branch, self_ty)?;
            let else_ = emit_expr(&else_branch.1, self_ty)?;
            Ok(format!("if {cond} then {then_} else {else_}"))
        }
        Expr::Match(m) => {
            let scrutinee = emit_expr(&m.expr, self_ty)?;
            let mut arms = Vec::with_capacity(m.arms.len());
            for arm in &m.arms {
                if arm.guard.is_some() {
                    return Err(GapReason::new(
                        Category::Other,
                        "match-arm guard (`if ...`) has no Mycelium equivalent (arm grammar has \
                         no guard slot)",
                    ));
                }
                let pat = map_pattern(&arm.pat)?;
                let body = emit_expr(&arm.body, self_ty)?;
                arms.push(format!("{pat} => {body}"));
            }
            Ok(format!("match {scrutinee} {{ {} }}", arms.join(", ")))
        }
        Expr::Binary(b) => {
            use syn::BinOp;
            let lhs = emit_expr(&b.left, self_ty)?;
            let rhs = emit_expr(&b.right, self_ty)?;
            match &b.op {
                BinOp::Eq(_) => Ok(format!("{lhs} == {rhs}")),
                BinOp::Ne(_) => Ok(format!("{lhs} != {rhs}")),
                BinOp::Lt(_) => Ok(format!("{lhs} < {rhs}")),
                BinOp::Gt(_) => Ok(format!("{lhs} > {rhs}")),
                BinOp::And(_) => Ok(format!("{lhs} && {rhs}")),
                BinOp::Or(_) => Ok(format!("{lhs} || {rhs}")),
                BinOp::BitAnd(_) => Ok(format!("{lhs} & {rhs}")),
                BinOp::BitOr(_) => Ok(format!("{lhs} | {rhs}")),
                BinOp::BitXor(_) => Ok(format!("{lhs} ^ {rhs}")),
                BinOp::Shl(_) => Ok(format!("{lhs} << {rhs}")),
                BinOp::Shr(_) => Ok(format!("{lhs} >> {rhs}")),
                BinOp::Add(_) => Ok(format!("{lhs} + {rhs}")),
                BinOp::Sub(_) => Ok(format!("{lhs} - {rhs}")),
                BinOp::Mul(_) => Ok(format!("{lhs} * {rhs}")),
                BinOp::Div(_) => Ok(format!("{lhs} / {rhs}")),
                BinOp::Rem(_) => Ok(format!("{lhs} % {rhs}")),
                // RFC-0025 §4.1: `<=`/`>=` glyphs are RETIRED; word forms `lte`/`gte` instead.
                BinOp::Le(_) => Ok(format!("lte({lhs}, {rhs})")),
                BinOp::Ge(_) => Ok(format!("gte({lhs}, {rhs})")),
                other => Err(GapReason::new(
                    Category::Other,
                    format!(
                        "unsupported/compound binary operator `{}`",
                        tokens_to_string(other)
                    ),
                )),
            }
        }
        Expr::Unary(u) => {
            let operand = emit_expr(&u.expr, self_ty)?;
            match &u.op {
                syn::UnOp::Neg(_) => Ok(format!("-{operand}")),
                syn::UnOp::Not(_) => Ok(format!("!{operand}")),
                _ => Err(GapReason::new(
                    Category::Other,
                    "unsupported unary operator (e.g. `*` deref has no equivalent in a \
                     value-semantic grammar)",
                )),
            }
        }
        Expr::Call(c) => {
            let func =
                match &*c.func {
                    Expr::Path(p) if p.qself.is_none() && p.path.segments.len() == 1 => p
                        .path
                        .segments
                        .last()
                        .map(|s| s.ident.to_string())
                        .ok_or_else(|| GapReason::new(Category::Other, "empty call-target path"))?,
                    Expr::Path(p) if p.qself.is_none() => {
                        // A qualified/associated-function call (`Type::method(...)`, e.g. Rust's
                        // widening bodies `i16::from(self)`). Mycelium calls are bare identifiers
                        // (`app_expr ::= primary ('(' args? ')')*`, `primary ::= ... | path`,
                        // `path ::= Ident ('.' Ident)*` — no `::`/qualifier form). An earlier
                        // iteration of this arm collapsed any path to its last segment, which for a
                        // *call target* fabricates a call to whatever that segment's name happens to
                        // be — e.g. `i16::from(self)` -> `from(self)`, and `from` is NOT a confirmed
                        // Mycelium builtin (grep of `docs/spec/grammar/mycelium.ebnf` finds it only in
                        // prose, never in a grammar production). There is no established Mycelium
                        // surface form for a Rust conversion-op/associated-fn call, so — mirroring
                        // `map::map_type`'s identical qualified-path decision — this is left an
                        // explicit gap rather than a fabricated call (G2/DN-34 §4).
                        return Err(GapReason::new(
                            Category::Other,
                            format!(
                            "qualified/associated-function call `{}` — no established Mycelium \
                             surface form for a Rust conversion-op body; emitting the bare \
                             last-segment name would fabricate a call (e.g. `from(...)` is not a \
                             Mycelium builtin)",
                            tokens_to_string(&*c.func)
                        ),
                        ));
                    }
                    _ => return Err(GapReason::new(
                        Category::Other,
                        "call target is not a simple path (e.g. a closure call) — no confirmed \
                         mapping",
                    )),
                };
            // M-1001: a call to a function whose name is a reserved word (e.g. a Rust `.swap()`
            // method or a `to(..)` helper) would emit un-parseable text; gap it (VR-5/G2).
            guard_ident(&func, "call target")?;
            let mut args = Vec::with_capacity(c.args.len());
            for a in &c.args {
                args.push(emit_expr(a, self_ty)?);
            }
            Ok(format!("{func}({})", args.join(", ")))
        }
        Expr::MethodCall(m) => {
            // Declared mapping decision: the grammar's `app_expr` has no postfix method-call
            // form (`primary ('(' args? ')')*` only) — desugar `recv.method(args)` to
            // `method(recv, args...)`, matching how `lib/std/cmp.myc`'s free functions
            // (`cmp`/`le`/`ge`/...) take the receiver as an ordinary first argument.
            guard_ident(&m.method.to_string(), "method call")?;
            let recv = emit_expr(&m.receiver, self_ty)?;
            let mut args = vec![recv];
            for a in &m.args {
                args.push(emit_expr(a, self_ty)?);
            }
            Ok(format!("{}({})", m.method, args.join(", ")))
        }
        Expr::Paren(p) => Ok(format!("({})", emit_expr(&p.expr, self_ty)?)),
        Expr::Reference(r) => {
            // Declared simplification: Mycelium is value-semantic (ADR-003) with no reference
            // type in this grammar fragment — `&expr`/`&mut expr` is treated as
            // reference-transparent and erased to its inner expression.
            emit_expr(&r.expr, self_ty)
        }
        Expr::Tuple(t) if t.elems.len() >= 2 => {
            let mut parts = Vec::with_capacity(t.elems.len());
            for e in &t.elems {
                parts.push(emit_expr(e, self_ty)?);
            }
            Ok(format!("({})", parts.join(", ")))
        }
        Expr::Tuple(t) if t.elems.is_empty() => Err(GapReason::new(
            Category::Other,
            "unit value `()` has no Mycelium literal",
        )),
        Expr::Tuple(_) => Err(GapReason::new(
            Category::Other,
            "single-element tuple `(x,)` has no Mycelium equivalent (tuple type requires arity \
             >= 2, M-826)",
        )),
        // An explicit-element array `[e1, e2, …]` maps to a Mycelium `ListLit` (grammar line 415:
        // `ListLit ::= '[' (expr (',' expr)*)? ']'`, constructs a `Seq{T, N}` — RFC-0032 D3). An
        // empty `[]` is a valid empty ListLit. Each element recurses through the guarded
        // `emit_expr`, so a non-expressible element gaps the whole array (never a partial list).
        Expr::Array(a) => {
            let mut elems = Vec::with_capacity(a.elems.len());
            for e in &a.elems {
                elems.push(emit_expr(e, self_ty)?);
            }
            Ok(format!("[{}]", elems.join(", ")))
        }
        // An array-repeat `[x; N]` has no Mycelium surface: `ListLit` (grammar line 415) enumerates
        // its elements and carries no repeat/count form — so this is an explicit, cited gap rather
        // than a fabricated expansion (which would also require evaluating `N`).
        Expr::Repeat(_) => Err(GapReason::new(
            Category::Other,
            "array-repeat expression `[x; N]` has no Mycelium equivalent — `ListLit ::= '[' (expr \
             (',' expr)*)? ']'` (grammar line 415) enumerates its elements and has no repeat form",
        )),
        Expr::Block(b) if b.label.is_none() => emit_block_as_expr(&b.block, self_ty),
        _ => Err(GapReason::new(
            Category::Other,
            format!("unsupported expression form `{}`", tokens_to_string(expr)),
        )),
    }
}

/// Translate one Rust pattern. Exhaustive `match` over `syn::Pat`; fallback arm errors.
///
/// **RFC-0041 §4.7 (W1):** guarded by the crate-wide recursion budget (`crate::gap::guarded`) —
/// self-recurses over unbounded/attacker-controlled pattern nesting (e.g. `Pat::Paren`/`Pat::Or`/
/// `Pat::TupleStruct`), so each call consumes one budget frame and refuses with a
/// `Category::RecursionBudget` gap rather than risking a host-stack overflow.
pub fn map_pattern(pat: &Pat) -> Result<String, GapReason> {
    guarded(|| map_pattern_inner(pat))
}

/// The recursion-guarded body of [`map_pattern`]. Recursive calls use the public `map_pattern`
/// name so each nested call re-enters the guard.
fn map_pattern_inner(pat: &Pat) -> Result<String, GapReason> {
    match pat {
        Pat::Wild(_) => Ok("_".to_string()),
        Pat::Ident(pi) if pi.by_ref.is_none() && pi.subpat.is_none() => {
            let name = pi.ident.to_string();
            guard_ident(&name, "match pattern binding/constructor")?;
            Ok(name)
        }
        Pat::Path(pp) if pp.qself.is_none() => {
            let seg = pp
                .path
                .segments
                .last()
                .ok_or_else(|| GapReason::new(Category::Other, "empty path pattern"))?;
            let name = seg.ident.to_string();
            guard_ident(&name, "match pattern constructor")?;
            Ok(name)
        }
        Pat::TupleStruct(pts) if pts.qself.is_none() => {
            let seg = pts.path.segments.last().ok_or_else(|| {
                GapReason::new(Category::Other, "empty tuple-struct pattern path")
            })?;
            guard_ident(&seg.ident.to_string(), "match pattern constructor")?;
            let mut elems = Vec::with_capacity(pts.elems.len());
            for e in &pts.elems {
                elems.push(map_pattern(e)?);
            }
            Ok(format!("{}({})", seg.ident, elems.join(", ")))
        }
        Pat::Lit(pl) => match &pl.lit {
            Lit::Bool(b) => Ok(if b.value { "True" } else { "False" }.to_string()),
            Lit::Int(i) => Ok(i.base10_digits().to_string()),
            _ => Err(GapReason::new(
                Category::Other,
                "unsupported literal pattern kind (only bool/int literal patterns map)",
            )),
        },
        Pat::Or(po) => {
            let mut alts = Vec::with_capacity(po.cases.len());
            for c in &po.cases {
                alts.push(map_pattern(c)?);
            }
            Ok(alts.join(" | "))
        }
        Pat::Tuple(pt) if pt.elems.len() >= 2 => {
            let mut elems = Vec::with_capacity(pt.elems.len());
            for e in &pt.elems {
                elems.push(map_pattern(e)?);
            }
            Ok(format!("({})", elems.join(", ")))
        }
        Pat::Paren(pp) => map_pattern(&pp.pat),
        Pat::Reference(pr) => map_pattern(&pr.pat),
        _ => Err(GapReason::new(
            Category::Other,
            format!("unsupported match pattern form `{}`", tokens_to_string(pat)),
        )),
    }
}

// ---------------------------------------------------------------------------------------------
// Top-level item emitters.
// ---------------------------------------------------------------------------------------------

/// `enum` -> `type_item` (`type Name = C1 | C2(T1, T2) | ...;`).
pub fn emit_enum(item: &ItemEnum) -> Result<Emitted, GapReason> {
    guard_ident(&item.ident.to_string(), "enum type name")?;
    let type_params = plain_type_params(&item.generics)?;
    let mut sub_gaps = Vec::new();
    let non_doc = non_doc_attrs(&item.attrs);
    if !non_doc.is_empty() {
        sub_gaps.push(GapReason::new(
            Category::DeriveAttr,
            format!(
                "dropped non-doc attribute(s) on enum `{}`: {}",
                item.ident,
                non_doc.join(" ")
            ),
        ));
    }
    let mut ctors = Vec::with_capacity(item.variants.len());
    for v in &item.variants {
        guard_ident(&v.ident.to_string(), "enum variant/constructor")?;
        if v.discriminant.is_some() {
            return Err(GapReason::new(
                Category::Other,
                format!(
                    "enum `{}` variant `{}` has an explicit discriminant — sum types are \
                     structural, not numeric",
                    item.ident, v.ident
                ),
            ));
        }
        match &v.fields {
            Fields::Unit => ctors.push(v.ident.to_string()),
            Fields::Unnamed(fields) => {
                let mut tys = Vec::with_capacity(fields.unnamed.len());
                for f in &fields.unnamed {
                    let mapped = map_type(&f.ty, None).map_err(|inner| {
                        GapReason::new(
                            Category::PayloadVariant,
                            format!(
                                "enum `{}` variant `{}` has a field type with no confirmed \
                                 mapping ({})",
                                item.ident, v.ident, inner.reason
                            ),
                        )
                    })?;
                    tys.push(mapped);
                }
                ctors.push(format!("{}({})", v.ident, tys.join(", ")));
            }
            Fields::Named(_) => {
                return Err(GapReason::new(
                    Category::PayloadVariant,
                    format!(
                        "enum `{}` variant `{}` uses named fields — `constructor ::= Ident \
                         ('(' type_ref (',' type_ref)* ')')?` has no named-field/record form",
                        item.ident, v.ident
                    ),
                ))
            }
        }
    }
    let params_text = if type_params.is_empty() {
        String::new()
    } else {
        format!("[{}]", type_params.join(", "))
    };
    let mut myc = String::new();
    for d in doc_lines(&item.attrs) {
        myc.push_str(&d);
        myc.push('\n');
    }
    myc.push_str(&format!(
        "type {}{} = {};",
        item.ident,
        params_text,
        ctors.join(" | ")
    ));
    Ok(Emitted {
        name: item.ident.to_string(),
        myc,
        sub_gaps,
    })
}

/// `struct` -> a single-constructor `type_item`. Unit and all-positional (`Fields::Unnamed`)
/// structs map; named-field structs/records have no grammar equivalent (KNOWN HARD GAP).
pub fn emit_struct(item: &ItemStruct) -> Result<Emitted, GapReason> {
    guard_ident(&item.ident.to_string(), "struct type/constructor name")?;
    let type_params = plain_type_params(&item.generics)?;
    let mut sub_gaps = Vec::new();
    let non_doc = non_doc_attrs(&item.attrs);
    if !non_doc.is_empty() {
        sub_gaps.push(GapReason::new(
            Category::DeriveAttr,
            format!(
                "dropped non-doc attribute(s) on struct `{}`: {}",
                item.ident,
                non_doc.join(" ")
            ),
        ));
    }
    let ctor = match &item.fields {
        Fields::Unit => item.ident.to_string(),
        Fields::Unnamed(fields) => {
            let mut tys = Vec::with_capacity(fields.unnamed.len());
            for f in &fields.unnamed {
                let mapped = map_type(&f.ty, None).map_err(|inner| {
                    GapReason::new(
                        Category::Struct,
                        format!(
                            "struct `{}` has a field type with no confirmed mapping ({})",
                            item.ident, inner.reason
                        ),
                    )
                })?;
                tys.push(mapped);
            }
            format!("{}({})", item.ident, tys.join(", "))
        }
        Fields::Named(_) => {
            return Err(GapReason::new(
                Category::Struct,
                format!(
                    "struct `{}` uses named fields — no record/product-type surface (only a \
                     single-ctor positional shape maps to `type_item`)",
                    item.ident
                ),
            ))
        }
    };
    let params_text = if type_params.is_empty() {
        String::new()
    } else {
        format!("[{}]", type_params.join(", "))
    };
    let mut myc = String::new();
    for d in doc_lines(&item.attrs) {
        myc.push_str(&d);
        myc.push('\n');
    }
    myc.push_str(&format!("type {}{} = {};", item.ident, params_text, ctor));
    Ok(Emitted {
        name: item.ident.to_string(),
        myc,
        sub_gaps,
    })
}

/// Top-level `fn` -> `fn_item`. No `self` (no enclosing impl/trait).
pub fn emit_fn(item: &ItemFn) -> Result<Emitted, GapReason> {
    guard_ident(&item.sig.ident.to_string(), "function name")?;
    check_fn_modifiers(&item.sig)?;
    let sig = map_signature(&item.sig.generics, &item.sig.inputs, &item.sig.output, None)?;
    let body = emit_block_as_expr(&item.block, None)?;
    let mut sub_gaps = Vec::new();
    let non_doc = non_doc_attrs(&item.attrs);
    if !non_doc.is_empty() {
        sub_gaps.push(GapReason::new(
            Category::DeriveAttr,
            format!(
                "dropped non-doc attribute(s) on fn `{}`: {}",
                item.sig.ident,
                non_doc.join(" ")
            ),
        ));
    }
    let myc = render_fn(
        &item.sig.ident.to_string(),
        &sig,
        &body,
        &doc_lines(&item.attrs),
    );
    Ok(Emitted {
        name: item.sig.ident.to_string(),
        myc,
        sub_gaps,
    })
}

/// `trait` -> `trait_item` (`trait Name { fn sig1; fn sig2; ... };`). Every method must have no
/// default body (`trait_item`'s `fn_sig` carries no body) and the trait must have no supertrait
/// bound (no supertrait syntax in the grammar). A method whose signature needs `Self`/`self`
/// still requires a concrete substitution the grammar has no slot for at trait-definition time,
/// so such methods fail their signature mapping (an honest, not a fabricated, "Self" binding).
pub fn emit_trait(item: &ItemTrait) -> Result<Emitted, GapReason> {
    guard_ident(&item.ident.to_string(), "trait name")?;
    if !item.supertraits.is_empty() {
        return Err(GapReason::new(
            Category::Trait,
            format!(
                "trait `{}` has supertrait bound(s) — trait_item grammar has no supertrait \
                 syntax (`'trait' Ident type_params? '{{' ...`)",
                item.ident
            ),
        ));
    }
    let type_params = plain_type_params(&item.generics)?;
    let mut sigs = Vec::with_capacity(item.items.len());
    for ti in &item.items {
        match ti {
            TraitItem::Fn(f) => {
                guard_ident(&f.sig.ident.to_string(), "trait method name")?;
                if f.default.is_some() {
                    return Err(GapReason::new(
                        Category::Trait,
                        format!(
                            "trait `{}` method `{}` has a default body — fn_sig carries no \
                             default implementation",
                            item.ident, f.sig.ident
                        ),
                    ));
                }
                check_fn_modifiers(&f.sig)?;
                let sig = map_signature(&f.sig.generics, &f.sig.inputs, &f.sig.output, None)
                    .map_err(|inner| {
                        GapReason::new(
                            Category::Trait,
                            format!(
                                "trait `{}` method `{}` signature has no confirmed mapping \
                                 (a trait-body `Self`/`self` has no concrete referent in this \
                                 grammar; {})",
                                item.ident, f.sig.ident, inner.reason
                            ),
                        )
                    })?;
                sigs.push(render_fn_sig(&f.sig.ident.to_string(), &sig));
            }
            TraitItem::Const(c) => {
                return Err(GapReason::new(
                    Category::AssocConst,
                    format!(
                        "trait `{}` has an associated const `{}` — trait_item body only allows \
                         fn_sig",
                        item.ident, c.ident
                    ),
                ))
            }
            TraitItem::Type(t) => {
                return Err(GapReason::new(
                    Category::Other,
                    format!(
                        "trait `{}` has an associated type `{}` — no equivalent in trait_item \
                         grammar",
                        item.ident, t.ident
                    ),
                ))
            }
            TraitItem::Macro(_) => {
                return Err(GapReason::new(
                    Category::MacroInvocation,
                    format!("trait `{}` body contains a macro invocation", item.ident),
                ))
            }
            _ => {
                return Err(GapReason::new(
                    Category::Other,
                    format!(
                        "trait `{}` contains an unrecognized trait-item form",
                        item.ident
                    ),
                ))
            }
        }
    }
    let params_text = if type_params.is_empty() {
        String::new()
    } else {
        format!("[{}]", type_params.join(", "))
    };
    let mut myc = String::new();
    for d in doc_lines(&item.attrs) {
        myc.push_str(&d);
        myc.push('\n');
    }
    // Each signature on its own indented line (readability, and consistency with the diff
    // harness's line-prefix `fn `/`type ` extraction — see `src/tests/diff.rs`).
    let sig_lines = sigs
        .iter()
        .map(|s| format!("  {s};"))
        .collect::<Vec<_>>()
        .join("\n");
    myc.push_str(&format!(
        "trait {}{} {{\n{}\n}};",
        item.ident, params_text, sig_lines
    ));
    Ok(Emitted {
        name: item.ident.to_string(),
        myc,
        sub_gaps: Vec::new(),
    })
}

/// `impl` -> `impl_item` (trait-instance or inherent form). Unlike enum/struct/trait (which bail
/// the whole item on the first unmappable feature), an impl block is emitted **partially**: each
/// method is attempted independently, a failing method becomes a sub-gap rather than voiding its
/// siblings, and the impl counts as "emitted" as long as at least one method landed. This is a
/// deliberate, documented asymmetry (Declared design choice) — impl methods are far more
/// independent of each other than, say, a trait's default-body/supertrait obligations are of its
/// sibling methods.
pub fn emit_impl(item: &ItemImpl) -> Result<Emitted, GapReason> {
    // impl_item has no generic-parameter declaration slot at all (unlike type_item/trait_item/
    // fn_item, which all carry `type_params?`) — so *any* impl-level generic parameter, bounded
    // or not, is a gap.
    if !item.generics.params.is_empty() {
        return Err(GapReason::new(
            Category::GenericBound,
            "impl block has generic parameter(s) — impl_item grammar has no generic-parameter \
             declaration slot",
        ));
    }
    if item.generics.where_clause.is_some() {
        return Err(GapReason::new(
            Category::WhereClause,
            "impl `where` clause has no Mycelium equivalent",
        ));
    }
    let self_ty_text = map_type(&item.self_ty, None).map_err(|inner| {
        GapReason::new(
            Category::Impl,
            format!(
                "impl target type `{}` has no confirmed mapping ({})",
                tokens_to_string(&*item.self_ty),
                inner.reason
            ),
        )
    })?;

    let (trait_name, trait_targs) = if let Some((_, trait_path, _)) = &item.trait_ {
        let seg = trait_path
            .segments
            .last()
            .ok_or_else(|| GapReason::new(Category::Impl, "impl trait path is empty"))?;
        guard_ident(&seg.ident.to_string(), "impl trait name")?;
        let targs =
            match &seg.arguments {
                PathArguments::None => Vec::new(),
                PathArguments::AngleBracketed(ab) => {
                    let mut v = Vec::with_capacity(ab.args.len());
                    for ga in &ab.args {
                        match ga {
                            GenericArgument::Type(t) => v.push(map_type(t, Some(&self_ty_text))?),
                            _ => return Err(GapReason::new(
                                Category::GenericBound,
                                "trait type argument is not a plain type (lifetime/const arg) — \
                                 no confirmed mapping",
                            )),
                        }
                    }
                    v
                }
                PathArguments::Parenthesized(_) => return Err(GapReason::new(
                    Category::GenericBound,
                    "parenthesized trait arguments (`Fn`-trait sugar) have no confirmed mapping",
                )),
            };
        (Some(seg.ident.to_string()), targs)
    } else {
        (None, Vec::new())
    };

    let mut sub_gaps = Vec::new();
    let mut method_bodies = Vec::new();
    for ii in &item.items {
        match ii {
            ImplItem::Fn(f) => {
                // M-1001: a reserved-word method name would emit un-parseable `fn <keyword>`; make
                // it a per-method sub-gap (keeping sibling methods independent), never emitted.
                if let Err(e) = guard_ident(&f.sig.ident.to_string(), "impl method name") {
                    sub_gaps.push(GapReason::new(
                        e.category,
                        format!("impl method `{}`: {}", f.sig.ident, e.reason),
                    ));
                    continue;
                }
                // DN-41 §2: `Narrow::narrow` is fallible (`Result<To, NarrowError>`) — no
                // `= expr fn_item` body can express a Result-returning refuse in this grammar
                // fragment, regardless of whether `Self`/the target type otherwise map. Intercept
                // before signature mapping so the recorded reason cites the real cause (DN-41)
                // rather than the incidental `Result<..>` generic-type-path gap that would
                // otherwise fire first and obscure it.
                if trait_name.as_deref() == Some("Narrow") && f.sig.ident == "narrow" {
                    sub_gaps.push(GapReason::new(
                        Category::Conversion,
                        "impl method `narrow`: DN-41 (docs/notes/DN-41-Width-Cast-Prim.md §2) \
                         specifies narrowing as fallible — `Result<To, NarrowError>`, refusing \
                         on an out-of-range/non-representable value — but this grammar \
                         fragment's `fn_item` body is a single `= expr` with no \
                         Result-returning surface to express that refuse; left an explicit gap \
                         rather than forced (VR-5)",
                    ));
                    continue;
                }
                if let Err(e) = check_fn_modifiers(&f.sig) {
                    sub_gaps.push(GapReason::new(
                        e.category,
                        format!("impl method `{}`: {}", f.sig.ident, e.reason),
                    ));
                    continue;
                }
                let width_cast_body = try_width_cast_widen_body(
                    trait_name.as_deref(),
                    &f.sig.ident.to_string(),
                    &self_ty_text,
                    &trait_targs,
                );
                match map_signature(
                    &f.sig.generics,
                    &f.sig.inputs,
                    &f.sig.output,
                    Some(&self_ty_text),
                ) {
                    Ok(sig) => {
                        let body_result = match &width_cast_body {
                            Some(body) => Ok(body.clone()),
                            None => emit_block_as_expr(&f.block, Some(&self_ty_text)),
                        };
                        match body_result {
                            Ok(body) => {
                                let non_doc = non_doc_attrs(&f.attrs);
                                if !non_doc.is_empty() {
                                    sub_gaps.push(GapReason::new(
                                        Category::DeriveAttr,
                                        format!(
                                            "dropped non-doc attribute(s) on method `{}`: {}",
                                            f.sig.ident,
                                            non_doc.join(" ")
                                        ),
                                    ));
                                }
                                let mut doc = doc_lines(&f.attrs);
                                if width_cast_body.is_some() {
                                    doc.push(
                                        "// Declared: body emitted via width_cast (DN-41 real \
                                         prim, docs/notes/DN-41-Width-Cast-Prim.md §2) — the \
                                         Binary{M} width witness is a synthesized all-zero BinLit \
                                         (RFC-0020 §Representation-tagged literals); unvalidated \
                                         by a Mycelium checker (crate-level Declared guarantee, \
                                         see src/lib.rs)."
                                            .to_string(),
                                    );
                                }
                                method_bodies.push(render_fn(
                                    &f.sig.ident.to_string(),
                                    &sig,
                                    &body,
                                    &doc,
                                ));
                            }
                            Err(e) => sub_gaps.push(GapReason::new(
                                e.category,
                                format!("impl method `{}` body: {}", f.sig.ident, e.reason),
                            )),
                        }
                    }
                    Err(e) => sub_gaps.push(GapReason::new(
                        e.category,
                        format!("impl method `{}` signature: {}", f.sig.ident, e.reason),
                    )),
                }
            }
            ImplItem::Const(c) => sub_gaps.push(GapReason::new(
                Category::AssocConst,
                format!("impl associated const `{}`", c.ident),
            )),
            ImplItem::Type(t) => sub_gaps.push(GapReason::new(
                Category::Other,
                format!("impl associated type `{}`", t.ident),
            )),
            ImplItem::Macro(_) => sub_gaps.push(GapReason::new(
                Category::MacroInvocation,
                "impl body contains a macro invocation".to_string(),
            )),
            _ => sub_gaps.push(GapReason::new(
                Category::Other,
                "impl contains an unrecognized impl-item form".to_string(),
            )),
        }
    }

    if method_bodies.is_empty() {
        let reason = if sub_gaps.is_empty() {
            "impl block has no items".to_string()
        } else {
            // Fold every sub-issue's own reason into the top-level gap's reason text. When an
            // impl fails wholesale (this arm), its `sub_gaps` are otherwise discarded — they are
            // only surfaced as separate `Gap` records via `emit::Emitted::sub_gaps` on the
            // *success* path (see `Outcome::Emitted` in `transpile.rs`). Folding them here keeps
            // this failure path never-silent too (G2): the specific reason (e.g. "no established
            // Mycelium surface form for `from(...)`") is never lost behind a generic count.
            let details = sub_gaps
                .iter()
                .map(|g| g.reason.as_str())
                .collect::<Vec<_>>()
                .join("; ");
            format!(
                "no member of this impl block could be transpiled ({} sub-issue(s)): {details}",
                sub_gaps.len()
            )
        };
        return Err(GapReason::new(Category::Impl, reason));
    }

    // Each method (and, when present, its own doc-comment lines) indented — same
    // readability/extraction rationale as `emit_trait`'s `sig_lines` above. `render_fn`'s output
    // may itself span multiple lines (doc comment + the `fn ...;` line), so indent every line,
    // not just the first.
    let body_text = method_bodies
        .iter()
        .map(|m| {
            m.lines()
                .map(|l| format!("  {l}"))
                .collect::<Vec<_>>()
                .join("\n")
        })
        .collect::<Vec<_>>()
        .join("\n");
    let mut myc = String::new();
    for d in doc_lines(&item.attrs) {
        myc.push_str(&d);
        myc.push('\n');
    }
    let name = if let Some(trait_name) = trait_name {
        let targs_text = if trait_targs.is_empty() {
            String::new()
        } else {
            format!("[{}]", trait_targs.join(", "))
        };
        myc.push_str(&format!(
            "impl {trait_name}{targs_text} for {self_ty_text} {{\n{body_text}\n}};"
        ));
        // Include type-args in the name so e.g. `impl Widen<u32> for bool` and
        // `impl Widen<u64> for bool` don't collide in `emitted_items`.
        format!("impl {trait_name}{targs_text} for {self_ty_text}")
    } else {
        myc.push_str(&format!("impl {self_ty_text} {{\n{body_text}\n}};"));
        format!("impl {self_ty_text}")
    };
    Ok(Emitted {
        name,
        myc,
        sub_gaps,
    })
}
