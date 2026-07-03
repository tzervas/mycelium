//! RFC-0041 §4.7/§5 — the guard-hole **census** (W0 safety net; RR-29 guard-hole inventory turned
//! into a tracked failing test for this crate's hole).
//!
//! Real repro: constructs a genuinely deep [`Node`] and calls the hole's entry point. Rust's default
//! stack-overflow handler aborts the process directly (never through panic/unwind), so this is not
//! `catch_unwind`-able — the test stays `#[ignore = "Wn"]`d; running it for real would crash the
//! whole test binary. When the named wave lands, drop the `#[ignore]` and the call must refuse
//! cleanly instead.

use mycelium_core::{Meta, Node, Payload, Provenance, Repr, Value};
use mycelium_lsp::project::llm_canonical;

/// A right-nested `Node::Let` chain, `n` deep — mirrors the shape `render_node`'s `Node::Let` arm
/// recurses on (`crates/mycelium-lsp/src/project.rs:44` `render_node`, dispatched from the public
/// [`llm_canonical`]).
fn deep_let(n: usize) -> Node {
    let byte = Value::new(
        Repr::Binary { width: 8 },
        Payload::Bits(vec![false; 8]),
        Meta::exact(Provenance::Root),
    )
    .expect("a well-formed Binary{8} const");
    let mut acc = Node::Const(byte.clone());
    for i in 0..n {
        acc = Node::Let {
            id: format!("x{i}"),
            bound: Box::new(Node::Const(byte.clone())),
            body: Box::new(acc),
        };
    }
    acc
}

/// Hole: `render_node` (`crates/mycelium-lsp/src/project.rs:44`), reached via the public
/// [`llm_canonical`] (RFC-0021 §4.6 `LlmCanonical` projection).
///
/// **Honesty (FLAG, VR-5):** `llm_canonical` returns a plain `String` — infallible today, so this
/// test cannot assert a "clean refusal" the way a `Result`-returning entry point can. It constructs
/// the real repro (the call itself, if unignored on a large enough `n`, is the SIGABRT) and documents
/// that RFC-0041 §4.7/§7 W1 is expected to route this projection through the shared work-step budget.
#[test]
#[ignore = "W1"] // RFC-0041 §4.7/§7 W1: frontend guard holes close.
fn render_node_deep_let_chain() {
    let deep = deep_let(200_000);
    let _ = llm_canonical(&deep);
}
