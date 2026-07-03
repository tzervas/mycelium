//! RFC-0041 §4.7/§5 — the guard-hole **census** (W0 safety net; RR-29 guard-hole inventory turned
//! into tracked failing tests, one per hole this crate owns).
//!
//! Real repros: each test constructs a genuinely deep [`Node`] and calls the hole's entry point.
//! Rust's default stack-overflow handler aborts the process directly (never through panic/unwind),
//! so none of this is `catch_unwind`-able — every test here stays `#[ignore = "Wn"]`d; running one
//! for real would crash the whole test binary. When the named wave lands, drop the `#[ignore]` and
//! the assertion must hold instead.

use mycelium_core::{Meta, Node, Payload, Provenance, Repr, Value};
use mycelium_mir_passes::emit::{count_occurrences, emit_owned};

/// A right-nested `Node::Let` chain, `n` deep, referencing `var` at its innermost leaf.
fn deep_let(n: usize, var: &str) -> Node {
    let byte = Value::new(
        Repr::Binary { width: 8 },
        Payload::Bits(vec![false; 8]),
        Meta::exact(Provenance::Root),
    )
    .expect("a well-formed Binary{8} const");
    let mut body = Node::Var(var.to_owned());
    for i in 0..n {
        body = Node::Let {
            id: format!("y{i}"),
            bound: Box::new(Node::Const(byte.clone())),
            body: Box::new(body),
        };
    }
    body
}

#[test]
#[ignore = "W1"] // RFC-0041 §4.7/§7 W1: frontend guard holes close ("mycelium-mir-passes").
fn emit_owned_deep_let_chain_refuses_cleanly() {
    // Hole: `emit_owned` (crates/mycelium-mir-passes/src/emit.rs:65) — recurses through `Let`'s
    // `bound` and `body`.
    let deep = deep_let(200_000, "x");
    let result = emit_owned(&deep);
    assert!(
        result.is_err(),
        "expected an explicit over-budget refusal, not success or a SIGABRT"
    );
}

/// Hole: `count_occurrences` (`crates/mycelium-mir-passes/src/emit.rs:186`).
///
/// **Honesty (FLAG, VR-5):** `count_occurrences` returns a plain `usize` — infallible today, so
/// this test cannot assert a "clean refusal". It constructs the real repro (the call itself, if
/// unignored on a large enough `n`, is the SIGABRT) and documents that RFC-0041 §4.7/§7 W1 is
/// expected to route this walk through the shared work-step budget.
#[test]
#[ignore = "W1"] // RFC-0041 §4.7/§7 W1.
fn count_occurrences_deep_let_chain() {
    let deep = deep_let(200_000, "x");
    let _ = count_occurrences(&"x".to_owned(), &deep);
}
