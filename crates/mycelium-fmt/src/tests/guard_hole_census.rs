//! RFC-0041 §4.7/§5 — the guard-hole **census** (W0 safety net; RR-29 guard-hole inventory turned
//! into a tracked failing test for this crate's hole).
//!
//! Real repro: builds a deep [`mycelium_l1::ast::Expr`] tree directly (bypassing the mycfmt public
//! text entry points on purpose — those parse through `mycelium_l1::parse`, which is ALREADY
//! depth-guarded at 256, so a text-based deep input never reaches this crate's own render family;
//! this crate's render recursion is reachable only from an AST built some other way, e.g. a future
//! non-text AST producer) and calls the private render helper directly (white-box, `use crate::*` —
//! CLAUDE.md test layout). Rust's default stack-overflow handler aborts the process directly (never
//! through panic/unwind), so this is not `catch_unwind`-able — the test stays `#[ignore = "Wn"]`d;
//! running it for real would crash the whole test binary.

use crate::*;
use mycelium_l1::ast::{Expr, Literal};

/// A right-nested `Expr::Let` chain, `n` deep.
fn deep_let(n: usize) -> Expr {
    let mut acc = Expr::Lit(Literal::Int(0));
    for i in 0..n {
        acc = Expr::Let {
            name: format!("x{i}"),
            ty: None,
            bound: Box::new(Expr::Lit(Literal::Int(0))),
            body: Box::new(acc),
        };
    }
    acc
}

/// Hole: the "fmt render family" — `render_expr_canonical` and its siblings
/// (`crates/mycelium-fmt/src/lib.rs:1629` `render_expr_canonical`, `:522` `render_flat`, `:573`
/// `render_item_flat`, `:604` `render_impl_flat`, `:1330` `render_body_with_comments`).
///
/// **Honesty (FLAG, VR-5):** `render_expr_canonical` returns a plain `String` — infallible today, so
/// this test cannot assert a "clean refusal". It constructs the real repro (the call itself, if
/// unignored on a large enough `n`, is the SIGABRT) and documents that RFC-0041 §4.7/§7 W1 is
/// expected to route this render family through the shared work-step budget.
#[test]
#[ignore = "W1"] // RFC-0041 §4.7/§7 W1: frontend guard holes close ("the mycelium-fmt render family").
fn render_expr_canonical_deep_let_chain() {
    let deep = deep_let(200_000);
    let _ = render_expr_canonical(&deep);
}
