//! RFC-0041 §4.7/§5 — the guard-hole **census** (W0 safety net; RR-29 guard-hole inventory turned
//! into a tracked failing test for this crate's hole).
//!
//! Real repro: builds a genuinely deep [`Node`] tree via the public [`Node::new`] constructor and
//! calls [`Node::walk`]. Rust's default stack-overflow handler aborts the process directly (never
//! through panic/unwind), so this is not `catch_unwind`-able — the test stays `#[ignore = "Wn"]`d;
//! running it for real would crash the whole test binary.

use mycelium_doc::ir::{Level, Node, Payload, Provenance};

/// A single-child chain of `Node`s, `n` deep.
fn deep_node(n: usize) -> Node {
    let mut acc = Node::new(
        "leaf",
        None,
        Some(Level::Minimal),
        Provenance {
            source: "guard-hole-census".to_owned(),
            line: 0,
        },
        Payload::Section,
        vec![],
    );
    for i in 0..n {
        acc = Node::new(
            format!("n{i}"),
            None,
            Some(Level::Minimal),
            Provenance {
                source: "guard-hole-census".to_owned(),
                line: 0,
            },
            Payload::Section,
            vec![acc],
        );
    }
    acc
}

/// Hole: `Node::walk` (`crates/mycelium-doc/src/ir.rs:310`) — depth-first pre-order visit,
/// unbounded recursion through `children`.
///
/// **Honesty (FLAG, VR-5):** `walk` takes a `&mut dyn FnMut(&Node)` callback and returns `()` —
/// infallible today, so this test cannot assert a "clean refusal". It constructs the real repro (the
/// call itself, if unignored on a large enough `n`, is the SIGABRT) and documents that RFC-0041
/// §4.7/§7 W1 is expected to route this walk through the shared work-step budget.
#[test]
#[ignore = "W1"] // RFC-0041 §4.7/§7 W1: frontend guard holes close ("mycelium-doc IR walkers").
fn node_walk_deep_chain() {
    let deep = deep_node(200_000);
    let mut count = 0usize;
    deep.walk(&mut |_n| count += 1);
    let _ = count;
}
