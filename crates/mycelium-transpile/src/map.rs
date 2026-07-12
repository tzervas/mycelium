//! The construct-mapping table: Rust `syn` types/paths -> Mycelium `type_ref` surface text, or
//! an explicit reason a mapping is not confirmed (never a guess — VR-5, G2).
//!
//! **Guarantee: `Declared`.** Every row here is a heuristic syn -> surface-text mapping verified
//! only against `docs/spec/grammar/mycelium.ebnf` (the grammar text), not against a Mycelium
//! parser or typechecker. Human-auditable: each row below carries a comment citing the grammar
//! fact it relies on.

use crate::gap::{guarded, Category, GapReason};
use quote::ToTokens;
use syn::{PathArguments, Type};

/// Render a `syn` node's tokens back to text, for gap snippets and unmapped-type messages only
/// (never used to build emitted `.myc` output — that always goes through the explicit mapping
/// functions in this module / `emit.rs`).
pub fn tokens_to_string<T: ToTokens>(node: &T) -> String {
    node.to_token_stream().to_string()
}

/// Map a Rust type to its Mycelium `type_ref` text.
///
/// `self_ty` supplies the substitution for `Self` inside an impl/trait body — `None` when there
/// is no enclosing impl/trait (a bare `Self` then has no referent and is a gap).
///
/// Returns `Err(GapReason)` when the type has no confirmed grammar surface. Confirmed rows (see
/// `docs/spec/grammar/mycelium.ebnf` §`base_type`):
/// - `bool` -> the ordinary named type `Bool` (used bare in `lib/std/cmp.myc`; base_type's
///   `Ident type_args?` arm covers an ordinary named type, so this assumes a kernel/prelude
///   `Bool` exists — Declared, not verified against a symbol table).
/// - unsigned integers (`u8`/`u16`/.../`u128`) -> `Binary{N}` (`base_type ::= 'Binary' '{' Int
///   '}'`).
/// - **P4/P5 (trx2, DN-99 §8 ENB-6 / M-1029 / ADR-028) — signed integers (`i8`/`i16`/`i32`/`i64`/
///   `i128`) -> `Binary{N}` at the SAME width as their unsigned counterpart.** The prior "unsigned
///   magnitude" doc comment here was the STALE basis for gapping these (mitigation #14
///   verify-first correction) — **ADR-028 (Accepted 2026-07-01) settles this**: "`Binary` **is**
///   the bitvector; 'signed integer' is an *interpretation* imposed by the op set (or a higher
///   typed view), not a property of the stored value." So `i32` and `u32` denote the exact same
///   Mycelium type/content-address (`Binary{32}`) — signedness is carried entirely by *which
///   operation* is applied (`add_s`/`sub_s`/`mul_s`/`neg_s`/`lt_s`; landed and confirmed `myc
///   check`-clean against the real `target/debug/myc-check`, this leaf's verify-first probe —
///   `crates/mycelium-l1/src/checkty.rs:8005-8040`, DN-72/M-767/M-887), never by a distinct
///   `Repr`. This is the "typed-view above kernel dispatches to signed ops" DN-99 row #44
///   describes ("ops need no work"); `crate::emit`'s `Expr::Binary`/`Expr::Unary` arms carry the
///   *transpile-time-only* signedness bookkeeping (never emitted into the `.myc` text itself —
///   `Binary` has no signed spelling) that picks the `_s`-suffixed op for a source-signed operand.
/// - **P4/P5 — `isize`/`usize` -> `Binary{64}`, a canonicalized, never-silent, FLAGged platform
///   width** (DN-99 §8 ENB-6 / row #22: "usize/uN -> domain Binary{N} + FLAG", "width choice
///   recorded never-silent"). 64 bits is the modern-platform default (every `myc check`-clean
///   probe this leaf ran was on a 64-bit host); it is a `Declared` context-free fallback, not a
///   domain-fitted choice — a call site with a *known* tighter domain may still choose a narrower
///   width by hand, as `lib/std/select.myc:76` already does (`Binary{8}` for a table index known
///   to be `0..=255`). `usize` carries no signed marker (it *is* unsigned); `isize` maps to the
///   same `Binary{64}` text but IS tracked signed by `crate::emit` for op routing (same mechanism
///   as the `i8`../`i128` case above).
/// - `f64` -> `Float` (`base_type ::= 'Float'`, `docs/spec/grammar/mycelium.ebnf:251` — a nullary
///   scalar-float type, "IEEE-754 binary64 only at introduction", ADR-040 FLAG-1/M-897; trx2 Lane C
///   Deliverable 2 verify-first correction — `myc check`-confirmed). `f32` -> gap (no confirmed
///   representation, `Float` being binary64-only). NOTE: `scalar` (`F16`/`BF16`/`F32`/`F64`) is a
///   *different*, Dense-only production (`Dense{N, scalar}`/`ambient_params`) — unrelated to the
///   bare `Float` value type.
/// - **P4/P5 — `char` -> `Binary{32}`** (a Unicode-scalar-value codepoint, per DN-99 §8 ENB-6 row
///   #45's "route char through the Bytes/std.text bridge" direction, resolved here toward the
///   **codepoint** idiom rather than a UTF-8 `Bytes` encoding — consistency with row #25's own
///   sanctioned char-*literal* idiom, "codepoint `0b…`/Int + `// 'x'` comment": a `char` value's
///   natural Mycelium spelling is the scalar codepoint it already is (Rust represents `char`
///   itself as a 4-byte scalar internally), not a variable-length UTF-8 byte sequence — `Bytes`
///   stays reserved for `String`/`str` (a *sequence* of codepoints). 32 bits comfortably covers
///   every Unicode Scalar Value (max `U+10FFFF`, 21 bits) with byte-aligned width, matching the
///   `u32`/`i32` precedent. `Declared` — no reified codepoint-domain check (`<= U+10FFFF`,
///   surrogate exclusion) is added by this mapping; a genuinely out-of-domain `char` value cannot
///   arise from real Rust source (the `char` type itself guarantees the invariant), so none is
///   needed here.
/// - `String`/`str`/`&str` -> `Bytes` (RFC-0033 §3.2: the dedicated, never-silent UTF-8 text repr;
///   grammar `base_type` line 250; a `"…"` StrLit lowers to the same `Repr::Bytes` value form —
///   checkty.rs:6669). Verified `myc check`-clean (DN-34 §8.14). `&str` is erased to `str` by the
///   shared-reference arm below, then mapped.
/// - `()` (unit) -> gap (no unit-value literal in the grammar's `literal`/`primary` productions).
/// - an ordinary zero-argument named type (`Ordering`, a same-crate type, etc.) -> passed through
///   as-is via `base_type`'s `Ident type_args?` arm.
/// - a tuple type of arity >= 2, all of whose elements map -> the grammar's tuple `type_ref` arm
///   (`'(' type_ref ',' type_ref (',' type_ref)* ')'`, M-826).
/// - a **shared** reference `&T` / `&'a T` -> the referent's mapping (the reference is *erased*).
///   Mycelium is value-semantic (ADR-003: no reference types; the grammar's `base_type`/`type_ref`
///   has no `&` form), so a shared borrow denotes the same `T` as the value — the type-position twin
///   of the reference-transparent erasure `emit.rs` already does on `&expr`/`&pat`, and how the
///   hand-port writes Rust `&Ordering` params as value `Ordering` (`lib/std/cmp.myc`). A referent
///   that itself has no mapping still gaps (its own precise reason surfaces — never a partial
///   emission). A **mutable** reference `&mut T` is NOT erased -> gap (in-place mutation has no
///   value-semantic correspondence — same stance as the `&mut self` receiver gap in
///   `emit::map_signature`).
/// - a single-segment named *generic application* (`Result<Duration, TimeErr>`, `Vec<u8>`,
///   `Option<T>`), all of whose angle-bracketed arguments are themselves mappable *types* ->
///   `Head[arg, …]` via `base_type ::= Ident type_args?` + `type_args ::= '[' type_ref (','
///   type_ref)* ']'` (grammar lines 258 + 265; RFC-0037 D1 uses `[]`, not `<>`). Refused as a gap
///   (never a partial emission) if the head is a reserved word, if any argument is a lifetime /
///   const-generic / associated-type binding-or-constraint, or if any argument type itself gaps.
/// - a *qualified* multi-segment path (`std::cmp::Ordering`, `crate::foo::Bar`) -> gap. Mycelium
///   `path`s are dot-joined and this module has no cross-nodule symbol table, so collapsing to
///   the last segment (as it did in an earlier iteration of this function) risked silently
///   conflating a foreign type with an unrelated local type of the same terminal name — a real
///   bug caught by inspecting this transpiler's own output on `std::cmp::Ordering` vs the local
///   `Ordering` (see the transpiler's report). Left an explicit gap rather than guessed (VR-5).
///
/// **RFC-0041 §4.7 (W1):** guarded by the crate-wide recursion budget (`crate::gap::guarded`) —
/// self-recurses over unbounded/attacker-controlled type nesting (a right-nested `Type::Tuple`),
/// so each call consumes one budget frame and refuses with a `Category::RecursionBudget` gap
/// rather than risking a host-stack overflow.
pub fn map_type(ty: &Type, self_ty: Option<&str>) -> Result<String, GapReason> {
    guarded(|| map_type_inner(ty, self_ty))
}

/// The recursion-guarded body of [`map_type`]. Recursive calls use the public `map_type` name so
/// each nested call re-enters the guard.
fn map_type_inner(ty: &Type, self_ty: Option<&str>) -> Result<String, GapReason> {
    // Routed through `crate::visit::TypeVisitor` (M-1041 Scope-A): the previous 3-shape
    // hand-written `match` now lives as `MapTypeVisitor`'s per-shape methods (below), reached via
    // the shared `crate::visit::walk_type` dispatcher (the same one `field_type_user_deps` now
    // uses, closing the drift risk this function's own doc named). Every method body is the
    // unmodified content of its former match arm (only bare `self_ty` references became
    // `self.self_ty`), so this is a pure relocation, not a behavior change (verified:
    // byte-identical `cargo test -p mycelium-transpile`).
    let mut visitor = MapTypeVisitor { self_ty };
    crate::visit::walk_type(ty, &mut visitor)
}

/// The `map_type_inner` translation, reified as a `crate::visit::TypeVisitor` (M-1041 Scope-A).
/// Each method below is the *unmodified* body of its former match arm — only the outer dispatch
/// moved to the shared `crate::visit::walk_type`, and the bare `self_ty` reference became
/// `self.self_ty` (a field instead of a function parameter, same value). No mapped type text and
/// no `GapReason` message changed.
struct MapTypeVisitor<'a> {
    self_ty: Option<&'a str>,
}

impl crate::visit::TypeVisitor for MapTypeVisitor<'_> {
    type Output = Result<String, GapReason>;

    fn fallback(&mut self, ty: &Type) -> Self::Output {
        Err(GapReason::new(
            Category::Other,
            format!("unsupported Rust type form `{}`", tokens_to_string(ty)),
        ))
    }

    fn visit_path(&mut self, ty: &Type, tp: &syn::TypePath) -> Self::Output {
        if tp.qself.is_none() && tp.path.segments.len() > 1 {
            return Err(GapReason::new(
                Category::Other,
                format!(
                    "qualified type path `{}` — collapsing to its last segment would risk colliding \
                     with an unrelated same-named local type (e.g. `std::cmp::Ordering` vs a local \
                     `Ordering`); left an explicit gap rather than guessed (VR-5)",
                    tokens_to_string(tp)
                ),
            ));
        }
        if tp.qself.is_some() {
            return self.fallback(ty);
        }
        let seg = tp
            .path
            .segments
            .last()
            .ok_or_else(|| GapReason::new(Category::Other, "empty type path".to_string()))?;
        let name = seg.ident.to_string();
        match name.as_str() {
            "Self" => self.self_ty.map(str::to_string).ok_or_else(|| {
                GapReason::new(
                    Category::Other,
                    "`Self` type with no enclosing impl/trait context",
                )
            }),
            "bool" => Ok("Bool".to_string()),
            "u8" => Ok("Binary{8}".to_string()),
            "u16" => Ok("Binary{16}".to_string()),
            "u32" => Ok("Binary{32}".to_string()),
            "u64" => Ok("Binary{64}".to_string()),
            "u128" => Ok("Binary{128}".to_string()),
            // P4/P5 (DN-99 §8 ENB-6 / M-1029 / ADR-028 — see this fn's doc for the full
            // verify-first correction): `Binary{N}` is sign-free (ADR-028 Accepted); a signed
            // integer maps to the SAME width `Binary{N}` as its unsigned counterpart. Signedness
            // lives entirely in which op the transpiler emits (`crate::emit`'s signed-operand
            // gate), never in this mapped type text.
            "i8" => Ok("Binary{8}".to_string()),
            "i16" => Ok("Binary{16}".to_string()),
            "i32" => Ok("Binary{32}".to_string()),
            "i64" => Ok("Binary{64}".to_string()),
            "i128" => Ok("Binary{128}".to_string()),
            // P4/P5 (DN-99 §8 ENB-6 row #22 — see this fn's doc): a canonicalized, FLAGged
            // platform width — `Binary{64}` — for both `usize` and `isize` (`isize`'s signedness
            // is tracked separately by `crate::emit`, exactly like the bare `i*` types above).
            "usize" | "isize" => Ok("Binary{64}".to_string()),
            // trx2 Lane C Deliverable 2 (verify-first correction, mitigation #14): the prior
            // "no confirmed base_type arm" reason for `f32`/`f64` was STALE — the grammar DOES
            // have a nullary `Float` base_type (`docs/spec/grammar/mycelium.ebnf:251`: "first-
            // class scalar float, IEEE-754 binary64 only at introduction (ADR-040 FLAG-1;
            // M-897) — nullary like Bytes"). `scalar` (`F16`/`BF16`/`F32`/`F64`) is a DIFFERENT,
            // Dense-only production (`Dense{N, scalar}`/`ambient_params`) — the earlier comment
            // conflated the two. Confirmed `myc check`-clean empirically: `fn f(x: Float) =>
            // Float = 1.5;` and `fn f(x: Float) => Binary{1} = flt_is_nan(x);` both check with
            // no import (`target/debug/myc`, `mycelium-proj.toml` `lang = "mycelium-0"`).
            // `Float` is explicitly "binary64 only at introduction" (a width extension is a
            // future, its-own-decision append — the grammar comment's own words), so `f64` maps
            // faithfully; `f32` still has no confirmed representation and stays a gap (never
            // silently widened/narrowed to `Float`, VR-5).
            "f64" => Ok("Float".to_string()),
            "f32" => Err(GapReason::new(
                Category::Other,
                "`f32` has no confirmed Mycelium representation — `Float` \
                 (docs/spec/grammar/mycelium.ebnf:251) is IEEE-754 binary64 only at \
                 introduction (ADR-040 FLAG-1/M-897); a width extension is a future, \
                 separately-decided append, never silently assumed (VR-5)",
            )),
            // P4/P5 (DN-99 §8 ENB-6 row #45 — see this fn's doc): the Unicode-scalar-value
            // codepoint idiom, `Binary{32}` — consistent with row #25's char-*literal* codepoint
            // convention. Unsigned (never tracked signed).
            "char" => Ok("Binary{32}".to_string()),
            // RFC-0033 §3.2 (grounded via tero, DN-34 §8.14): `Bytes` is the language's
            // *dedicated, never-silent UTF-8* text repr (grammar `base_type` line 250,
            // "first-class byte string"; a `"…"` StrLit lowers to the same `Repr::Bytes` value
            // form — checkty.rs:6669, M-910/M-911). So Rust `String`/`str` map onto `Bytes`
            // faithfully: both denote an owned UTF-8 text value under value semantics (ADR-003),
            // and the earlier "not confirmed equivalent" hedge is resolved by §3.2. Verified
            // `myc check`-clean (a `Bytes`-typed field/param/return and a `"…"` literal all pass
            // — DN-34 §8.14 verify-first). This is the type-position twin of the string-literal
            // value emission `emit.rs` already performs (`Lit::Str` -> `StrLit`). Graded
            // `Declared` like every row here (grammar-text- + oracle-verified, not proven).
            "String" | "str" => Ok("Bytes".to_string()),
            _ if matches!(seg.arguments, PathArguments::None) => {
                // M-1001: an ordinary named type passed through as-is — but if its name is a
                // Mycelium reserved word (e.g. a Rust type literally named `Binary`/`Float`), the
                // bare identifier would lex as a keyword and fail to parse. Gap it (never emit
                // un-parseable text) rather than guess a rename (VR-5/G2).
                crate::reserved::guard_ident(&name, "type name")?;
                Ok(name)
            }
            // A single-segment named *generic application* (`Result<Duration, TimeErr>`,
            // `Vec<u8>`, `Option<T>`). Confirmed surface: `base_type ::= Ident type_args?` with
            // `type_args ::= '[' type_ref (',' type_ref)* ']'`
            // (docs/spec/grammar/mycelium.ebnf lines 258 + 265 — RFC-0037 D1: type arguments in
            // square brackets, not `<…>`). Every scalar/gapped builtin (`bool`/`u8`.../`String`/
            // …) already matched an earlier arm, so a generic application is *never* mapped onto
            // a `Bool`/`Binary{N}`/`String` head here — only ordinary named heads reach this arm
            // (they fall through the builtin name matches, exactly as the bare-named arm above).
            // Graded `Declared` like every row in this module (grammar-text-verified only).
            _ => match &seg.arguments {
                PathArguments::AngleBracketed(ab) => {
                    // Head maps exactly as the bare-named arm does — a reserved-word head still
                    // gaps (never emit un-lexable text; VR-5/G2), before any argument work.
                    crate::reserved::guard_ident(&name, "type name")?;
                    let mut args = Vec::with_capacity(ab.args.len());
                    for arg in &ab.args {
                        match arg {
                            // Recurse through the *public* `map_type` (not `_inner`) so the
                            // recursion budget re-arms per nested application — same pattern as
                            // the tuple arm below — and, as there, a type argument that itself
                            // gaps propagates its own precise `GapReason` unchanged (`?`), never
                            // a partial emission.
                            syn::GenericArgument::Type(t) => args.push(map_type(t, self.self_ty)?),
                            // A lifetime / const-generic / associated-type binding-or-constraint
                            // (or any future non-`Type` `GenericArgument`) has no `type_ref`-
                            // shaped `type_args` surface (line 265 admits only `type_ref`s), so
                            // refuse the whole application rather than drop the argument (G2).
                            other => {
                                return Err(GapReason::new(
                                    Category::GenericBound,
                                    format!(
                                        "generic type path `{}` — type argument `{}` is not a \
                                         type (lifetime / const-generic / associated-type \
                                         binding-or-constraint); `type_args` admits only \
                                         type_refs, so left an explicit gap (VR-5)",
                                        tokens_to_string(tp),
                                        tokens_to_string(other)
                                    ),
                                ));
                            }
                        }
                    }
                    // `type_args ::= '[' type_ref (',' type_ref)* ']'` requires >= 1 type_ref;
                    // an empty `<>` has no confirmed surface.
                    if args.is_empty() {
                        return Err(GapReason::new(
                            Category::GenericBound,
                            format!(
                                "generic type path `{}` — empty type-argument list has no \
                                 confirmed `type_args` surface (requires >= 1 type_ref)",
                                tokens_to_string(tp)
                            ),
                        ));
                    }
                    Ok(format!("{name}[{}]", args.join(", ")))
                }
                // Non-angle-bracketed arguments (e.g. an `Fn(..)`-trait parenthesized form) —
                // no confirmed grammar surface; left an explicit gap.
                _ => Err(GapReason::new(
                    Category::GenericBound,
                    format!(
                        "generic type path `{}` — type-argument mapping not confirmed",
                        tokens_to_string(tp)
                    ),
                )),
            },
        }
    }

    fn visit_tuple(&mut self, ty: &Type, t: &syn::TypeTuple) -> Self::Output {
        if t.elems.is_empty() {
            Err(GapReason::new(
                Category::Other,
                "unit type `()` has no representable value in this grammar fragment",
            ))
        } else if t.elems.len() >= 2 {
            let mut parts = Vec::with_capacity(t.elems.len());
            for elem in &t.elems {
                parts.push(map_type(elem, self.self_ty)?);
            }
            Ok(format!("({})", parts.join(", ")))
        } else {
            // A single-element tuple type `(T,)` has no dedicated arm in the pre-refactor `match`
            // either (only `is_empty()`/`len() >= 2` were named) — it fell to the generic `_`
            // catch-all, so it does here too (`self.fallback`).
            self.fallback(ty)
        }
    }

    fn visit_reference(&mut self, ty: &Type, r: &syn::TypeReference) -> Self::Output {
        // A **shared** reference type `&T` / `&'a T` has no Mycelium reference-type surface — the
        // grammar's `type_ref`/`base_type` (docs/spec/grammar/mycelium.ebnf §`base_type`) admits no
        // `&` form, and Mycelium is value-semantic (ADR-003: there are no reference types). Under
        // value semantics a shared borrow and the value it borrows denote the *same* `T`, so the
        // reference is **erased** and its referent type mapped. This is the type-position analogue of
        // the reference-transparent erasure `emit.rs` already performs on `&expr` (`Expr::Reference`)
        // and `&pat` (`Pat::Reference`), and it is exactly how the hand-port renders Rust `&Ordering`
        // params as value `Ordering` (`lib/std/cmp.myc`'s `fn cmp(a: Ordering, b: Ordering)` for the
        // Rust `fn cmp(&self, other: &Ordering)`). The lifetime, if any, is erased with the reference
        // (lifetimes have no grammar surface). Recurse through the *public* `map_type` so the
        // recursion budget re-arms per level (same pattern as the tuple arm) — and a referent type
        // that itself has no confirmed mapping propagates its own precise `GapReason` unchanged (`?`),
        // never a partial emission (so `&str`/`&[u8]`/`&dyn T` surface their *referent's* real
        // blocker, not the reference; VR-5/G2).
        if r.mutability.is_none() {
            map_type(&r.elem, self.self_ty)
        } else {
            // A **mutable** reference `&mut T` is NOT erased. In-place mutation through a `&mut` has
            // no value-semantic correspondence (ADR-003) — the same stance the `&mut self` receiver
            // already takes in `emit::map_signature` — so erasing it to a plain value type would
            // silently drop the mutation. Left an explicit gap rather than a misrepresentation
            // (VR-5/G2).
            Err(GapReason::new(
                Category::Other,
                format!(
                    "`{}` is a mutable reference `&mut T` — in-place mutation through a borrow has no \
                     value-semantic correspondence (ADR-003; cf. the `&mut self` receiver gap), so it \
                     is left an explicit gap rather than silently erased to a value type (VR-5)",
                    tokens_to_string(ty)
                ),
            ))
        }
    }
}

/// For the M-1006 **resolvability fixpoint** (`transpile::resolvable_type_names`): collect the bare,
/// single-segment **user** type names `ty` references (the ones [`map_type`] passes through *as-is* —
/// i.e. not builtins), pushing them into `out`. Returns `false` when `ty` has **no** [`map_type`]
/// mapping at all (an unmappable field ⇒ its record can never be resolvable — consistent with
/// `map_type` gapping the field). Builtins (`bool`, `u8..u128`) and tuples/shared-refs/generic-apps
/// of mappables are traversed for their nested user names but are not themselves deps.
///
/// This deliberately **mirrors [`map_type`]'s mappable shapes**; if the two drift, the only cost is a
/// *missed* emission (a struct conservatively left gapped) — never an unsound one (VR-5): the gate is
/// one-sided (it can only *withhold* an emission, so a stale mirror is safe, just less generous).
pub(crate) fn field_type_user_deps(ty: &Type, out: &mut Vec<String>) -> bool {
    // Routed through `crate::visit::TypeVisitor` (M-1041 Scope-A) — the same shared
    // `crate::visit::walk_type` dispatcher `map_type_inner` now uses, closing the drift risk this
    // function's own doc comment (above) named explicitly ("this deliberately mirrors
    // `map_type`'s mappable shapes; if the two drift…"). `FieldDepsVisitor`'s methods are the
    // unmodified bodies of this function's former match arms (only `out` became `self.out`).
    crate::visit::walk_type(ty, &mut FieldDepsVisitor { out })
}

/// The `field_type_user_deps` fixpoint walk, reified as a `crate::visit::TypeVisitor` (M-1041
/// Scope-A). Each method is the *unmodified* body of its former match arm.
struct FieldDepsVisitor<'a> {
    out: &'a mut Vec<String>,
}

impl crate::visit::TypeVisitor for FieldDepsVisitor<'_> {
    type Output = bool;

    fn fallback(&mut self, _ty: &Type) -> Self::Output {
        false
    }

    fn visit_path(&mut self, ty: &Type, tp: &syn::TypePath) -> Self::Output {
        if !(tp.qself.is_none() && tp.path.segments.len() == 1) {
            // Qualified/multi-segment path (or an empty-`qself`-carrying one): `map_type` gaps
            // it (unmappable) — mirrors that function's `Type::Path(_) => false` catch-all.
            return self.fallback(ty);
        }
        let seg = match tp.path.segments.last() {
            Some(s) => s,
            None => return false,
        };
        let name = seg.ident.to_string();
        match name.as_str() {
            // Builtins `map_type` maps directly — mappable, but contribute no user dep.
            // `String`/`str` now map to `Bytes` (RFC-0033 §3.2 — DN-34 §8.14), so they join the
            // builtins here: a `String`-typed field no longer withholds its struct's emission.
            // `f64` now maps to `Float` (trx2 Lane C Deliverable 2 — see `map_type`'s doc); it
            // joins the builtins here too, for the identical reason. `i8..i128`/`isize`/`usize`/
            // `char` now map too (P4/P5, DN-99 §8 ENB-6 — see `map_type`'s doc) and join here.
            "bool" | "u8" | "u16" | "u32" | "u64" | "u128" | "String" | "str" | "f64" | "i8"
            | "i16" | "i32" | "i64" | "i128" | "isize" | "usize" | "char" => {
                matches!(seg.arguments, PathArguments::None)
            }
            // Shapes `map_type` gaps outright ⇒ unmappable field.
            "Self" | "f32" => false,
            _ => {
                // A reserved-word type name fails to lex ⇒ `map_type` gaps it (unmappable).
                if crate::reserved::is_reserved(&name) {
                    return false;
                }
                match &seg.arguments {
                    PathArguments::None => {
                        self.out.push(name);
                        true
                    }
                    PathArguments::AngleBracketed(ab) => {
                        self.out.push(name);
                        !ab.args.is_empty()
                            && ab.args.iter().all(|a| match a {
                                syn::GenericArgument::Type(t) => field_type_user_deps(t, self.out),
                                _ => false,
                            })
                    }
                    _ => false,
                }
            }
        }
    }

    fn visit_tuple(&mut self, _ty: &Type, t: &syn::TypeTuple) -> Self::Output {
        if t.elems.is_empty() {
            false
        } else if t.elems.len() >= 2 {
            t.elems.iter().all(|e| field_type_user_deps(e, self.out))
        } else {
            // A single-element tuple type `(T,)` fell to the generic `_ => false` catch-all
            // pre-refactor too (no dedicated arm) — `fallback` reproduces that.
            false
        }
    }

    fn visit_reference(&mut self, _ty: &Type, r: &syn::TypeReference) -> Self::Output {
        if r.mutability.is_none() {
            field_type_user_deps(&r.elem, self.out)
        } else {
            false
        }
    }
}
