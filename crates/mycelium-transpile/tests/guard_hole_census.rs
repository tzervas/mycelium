//! RFC-0041 §4.7/§5 — the guard-hole **census** (W0 safety net; RR-29 guard-hole inventory turned
//! into tracked failing tests, one per hole this crate owns).
//!
//! Real repros: each test builds genuinely deep Rust source text via `syn::parse_str` (a real,
//! user-triggerable input — the transpiler's whole job is ingesting third-party Rust) and calls the
//! hole's entry point. Rust's default stack-overflow handler aborts the process directly (never
//! through panic/unwind), so none of this is `catch_unwind`-able — every test here stays
//! `#[ignore = "Wn"]`d; running one for real would crash the whole test binary. When the named wave
//! lands, drop the `#[ignore]` and the assertion must hold instead.

use mycelium_transpile::emit::{emit_expr, map_pattern};
use mycelium_transpile::map::map_type;

/// `n` levels of parenthesized nesting: `(((…(1)…)))`.
fn deep_parens(n: usize) -> String {
    format!("{}1{}", "(".repeat(n), ")".repeat(n))
}

/// `n`-deep right-nested 2-tuple type: `(u8, (u8, (u8, …)))`. `map_type` has no `Type::Paren` arm
/// (a bare parenthesized type falls through its catch-all `_ => Err(..)`, never recursing), so a
/// right-nested `Type::Tuple` (its one recursive arm, `map.rs:127`) is the real repro shape instead.
fn deep_type_tuple(n: usize) -> String {
    format!("{}u8{}", "(u8, ".repeat(n), ")".repeat(n))
}

/// `n` levels of nested tuple-pattern parens: `(((…(x)…)))`.
fn deep_pattern_parens(n: usize) -> String {
    format!("{}x{}", "(".repeat(n), ")".repeat(n))
}

#[test]
#[ignore = "W1"] // RFC-0041 §4.7/§7 W1: frontend guard holes close ("mycelium-transpile").
fn emit_expr_deep_paren_refuses_cleanly() {
    // Hole: `emit_expr` (crates/mycelium-transpile/src/emit.rs:405) — recurses through
    // `Expr::Paren`'s inner expression.
    let src = deep_parens(200_000);
    let expr: syn::Expr = syn::parse_str(&src).expect("deeply-parenthesized Rust still parses");
    let result = emit_expr(&expr, None);
    assert!(
        result.is_err(),
        "expected an explicit over-budget GapReason refusal, not success or a SIGABRT"
    );
}

#[test]
#[ignore = "W1"] // RFC-0041 §4.7/§7 W1.
fn map_type_deep_tuple_refuses_cleanly() {
    // Hole: `map_type` (crates/mycelium-transpile/src/map.rs:49) — recurses through `Type::Tuple`
    // elements (`map.rs:127`), the crate's own real repro shape (see `deep_type_tuple` doc comment).
    let src = deep_type_tuple(200_000);
    let ty: syn::Type = syn::parse_str(&src).expect("a right-nested 2-tuple type still parses");
    let result = map_type(&ty, None);
    assert!(
        result.is_err(),
        "expected an explicit over-budget GapReason refusal, not success or a SIGABRT"
    );
}

#[test]
#[ignore = "W1"] // RFC-0041 §4.7/§7 W1.
fn map_pattern_deep_paren_refuses_cleanly() {
    // Hole: `map_pattern` (crates/mycelium-transpile/src/emit.rs:600) — recurses through
    // `Pat::Paren`.
    let src = deep_pattern_parens(200_000);
    // `Pat` has no direct `Parse` impl (it needs disambiguation re: a leading `|`) — go through the
    // `Parser` trait's `Pat::parse_single`, syn 2's documented way to parse a single bare pattern.
    let pat: syn::Pat = syn::parse::Parser::parse_str(syn::Pat::parse_single, &src)
        .expect("deeply-parenthesized Rust pattern still parses");
    let result = map_pattern(&pat);
    assert!(
        result.is_err(),
        "expected an explicit over-budget GapReason refusal, not success or a SIGABRT"
    );
}
