//! RFC-0041 §4.7/§5 — the guard-hole **census** (W0 safety net; RR-29 guard-hole inventory turned
//! into a tracked failing test for this crate's hole).
//!
//! Real repro: builds a genuinely deep [`Node`] tree via the public [`Node::new`] constructor and
//! calls [`Node::walk`]. Rust's default stack-overflow handler aborts the process directly (never
//! through panic/unwind), so this was not `catch_unwind`-able while the hole was open.
//!
//! **W1 (closed):** `Node::walk` now runs on
//! [`mycelium_workstack::ensure_sufficient_stack`]'s grown worker stack (RFC-0041 §4.7), so this
//! `200_000`-deep chain walks to clean completion instead of `SIGABRT`ing — the test below asserts
//! exactly that (no longer `#[ignore]`d).

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

/// Hole (closed, W1): `Node::walk` (`crates/mycelium-doc/src/ir.rs`) — depth-first pre-order visit,
/// recursion through `children`, now guarded by
/// [`mycelium_workstack::ensure_sufficient_stack`] at its single outermost entry.
///
/// **Honesty (VR-5):** `walk` takes a `&mut (dyn FnMut(&Node) + Send)` callback and returns `()` —
/// infallible, so this cannot assert a "clean refusal" (there is no refusal path — W1 closes the
/// host-stack hole, it does not add a new depth ceiling). Instead it asserts **clean completion**: the
/// same 200,000-deep chain that used to `SIGABRT` now walks to the end and visits every node exactly
/// once.
///
/// **FLAG (new, distinct hole — out of W1 scope):** dropping this same 200,000-deep chain normally
/// (derived, recursive `Drop` through `children: Vec<Node>`) overflows the *caller's* stack on its own
/// — confirmed empirically here — independent of `walk`. This is the doc-IR-`Node` analogue of the
/// `mycelium-core`/`mycelium-l1` recursive-`Drop` bomb RFC-0041 §4.5/W3 already tracks (research/29
/// §"drop-bomb"), but RFC-0041's W3 scope is those kernel/L1 value types, **not** `mycelium-doc`'s own
/// `ir::Node` — this instance is not yet in any tracked issue. `std::mem::forget` sidesteps it here so
/// this test exercises exactly the `walk` fix (never silently papering over the finding — flagged for
/// a future issue, not fixed in this leaf).
#[test]
fn node_walk_deep_chain() {
    let deep = deep_node(200_000);
    let mut count = 0usize;
    deep.walk(&mut |_n| count += 1);
    assert_eq!(count, 200_001);
    // See the FLAG above: a normal `drop(deep)` here overflows the stack via `Node`'s derived
    // recursive `Drop` — a separate, untracked hole this W1 leaf does not fix. `forget` keeps this
    // test scoped to `walk`.
    std::mem::forget(deep);
}
