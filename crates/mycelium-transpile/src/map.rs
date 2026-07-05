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
///   '}'`). `lib/std/cmp.myc`'s own comments describe `Binary{N}` as **unsigned magnitude** —
///   so *signed* integers (`i8`.../`isize`) are intentionally NOT mapped here (would misrepresent
///   twos-complement semantics as an unsigned-magnitude representation); they are a gap.
/// - `isize`/`usize` -> gap (platform-dependent width has no fixed `Binary{N}`).
/// - `f32`/`f64`/`char`/`String`/`str` -> gap (no confirmed base_type arm for any of these in
///   this grammar fragment; `scalar` only appears inside `Dense{N, scalar}`/`ambient_params`,
///   never as a bare value type).
/// - `()` (unit) -> gap (no unit-value literal in the grammar's `literal`/`primary` productions).
/// - an ordinary zero-argument named type (`Ordering`, a same-crate type, etc.) -> passed through
///   as-is via `base_type`'s `Ident type_args?` arm.
/// - a tuple type of arity >= 2, all of whose elements map -> the grammar's tuple `type_ref` arm
///   (`'(' type_ref ',' type_ref (',' type_ref)* ')'`, M-826).
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
    match ty {
        Type::Path(tp) if tp.qself.is_none() && tp.path.segments.len() > 1 => Err(GapReason::new(
            Category::Other,
            format!(
                "qualified type path `{}` — collapsing to its last segment would risk colliding \
                 with an unrelated same-named local type (e.g. `std::cmp::Ordering` vs a local \
                 `Ordering`); left an explicit gap rather than guessed (VR-5)",
                tokens_to_string(tp)
            ),
        )),
        Type::Path(tp) if tp.qself.is_none() => {
            let seg =
                tp.path.segments.last().ok_or_else(|| {
                    GapReason::new(Category::Other, "empty type path".to_string())
                })?;
            let name = seg.ident.to_string();
            match name.as_str() {
                "Self" => self_ty.map(str::to_string).ok_or_else(|| {
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
                "i8" | "i16" | "i32" | "i64" | "i128" => Err(GapReason::new(
                    Category::Other,
                    format!(
                        "signed integer `{name}` — Binary{{N}} is documented unsigned-magnitude \
                         (lib/std/cmp.myc); mapping a signed type onto it would misrepresent \
                         twos-complement semantics, so this is left an explicit gap rather than \
                         guessed (VR-5)"
                    ),
                )),
                "isize" | "usize" => Err(GapReason::new(
                    Category::Other,
                    format!(
                        "`{name}` has a platform-dependent width; no fixed Binary{{N}} mapping"
                    ),
                )),
                "f32" | "f64" => Err(GapReason::new(
                    Category::Other,
                    format!(
                        "`{name}` — `scalar` only appears inside Dense{{N,scalar}}/ambient_params \
                         in the grammar, never as a bare value type; no confirmed base_type arm"
                    ),
                )),
                "char" => Err(GapReason::new(
                    Category::Other,
                    "`char` has no confirmed base_type arm in this grammar fragment",
                )),
                "String" | "str" => Err(GapReason::new(
                    Category::Other,
                    "`String`/`str` — `Bytes` exists in base_type but is not confirmed \
                     equivalent to a UTF-8 text type",
                )),
                _ if matches!(seg.arguments, PathArguments::None) => Ok(name),
                _ => Err(GapReason::new(
                    Category::GenericBound,
                    format!(
                        "generic type path `{}` — type-argument mapping not confirmed",
                        tokens_to_string(tp)
                    ),
                )),
            }
        }
        Type::Tuple(t) if t.elems.is_empty() => Err(GapReason::new(
            Category::Other,
            "unit type `()` has no representable value in this grammar fragment",
        )),
        Type::Tuple(t) if t.elems.len() >= 2 => {
            let mut parts = Vec::with_capacity(t.elems.len());
            for elem in &t.elems {
                parts.push(map_type(elem, self_ty)?);
            }
            Ok(format!("({})", parts.join(", ")))
        }
        _ => Err(GapReason::new(
            Category::Other,
            format!("unsupported Rust type form `{}`", tokens_to_string(ty)),
        )),
    }
}
