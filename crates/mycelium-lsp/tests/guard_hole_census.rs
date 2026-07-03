//! RFC-0041 §4.7/§5 — the guard-hole **census** (W0 safety net; RR-29 guard-hole inventory turned
//! into a tracked failing test for this crate's hole).
//!
//! Real repro: constructs a genuinely deep [`Node`] and calls the hole's entry point. Rust's default
//! stack-overflow handler aborts the process directly (never through panic/unwind), so this is not
//! `catch_unwind`-able — the test stayed `#[ignore = "W1"]`d until the wave landed. **RFC-0041 §4.7
//! W1 (this crate):** `llm_canonical` now wraps `render_node` in
//! [`mycelium_workstack::ensure_sufficient_stack`] (`crates/mycelium-lsp/src/project.rs`), so the
//! deep render below runs on the grown 256 MiB worker stack and completes cleanly instead of
//! aborting the test binary — the `#[ignore]` is dropped and the census now asserts clean completion.

use mycelium_core::{Meta, Node, Payload, Provenance, Repr, Value};
use mycelium_lsp::project::llm_canonical;
use mycelium_workstack::{ensure_sufficient_stack, RecursionBudget};

/// A right-nested `Node::Let` chain, `n` deep — mirrors the shape `render_node`'s `Node::Let` arm
/// recurses on (`crates/mycelium-lsp/src/project.rs` `render_node`, dispatched from the public
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

/// Closed hole: `render_node` (`crates/mycelium-lsp/src/project.rs`), reached via the public
/// [`llm_canonical`] (RFC-0021 §4.6 `LlmCanonical` projection), now runs on the grown host stack.
///
/// **Honesty (VR-5):** `llm_canonical` returns a plain `String` — infallible by signature, so this
/// asserts **clean completion** (a well-formed, non-empty render), not a `Result` refusal the way a
/// fallible entry point could assert. The W1 fix is "never a host-stack abort", not "reject deep
/// input" — routing this projection through a depth-refusal budget is later work (W2+, §4.2/§7); the
/// census here documents exactly that scope.
///
/// **FLAG (test-harness artifact, out of scope for this leaf):** `Node`'s `Drop` glue is itself
/// recursive (one frame per nesting level, compiler-generated for the `Box<Node>` fields), so
/// dropping a deep fixture on the test harness's default per-test thread stack would overflow
/// *that* — a hazard of this synthetic fixture's construction/teardown, not of the `render_node`
/// guard hole under test (a real LSP host never constructs/drops a whole document tree in one
/// recursive call the way this fixture builder's `Drop` does). The whole test body — build, render,
/// assert, drop — runs inside the same [`ensure_sufficient_stack`] worker the production fix uses,
/// so the fixture's construction and teardown get the same 256 MiB headroom as the render.
///
/// **Depth choice:** `n = 20_000` is ~78× the L1 parser's 256-frame depth guard (and ~312× the
/// evaluator's 64) — comfortably past the old unguarded default host-stack size (a few MiB), which
/// is what made the original `#[ignore]`d repro SIGABRT. It is *not* `RecursionBudget::DEFAULT_DEPTH_LIMIT`
/// (4096) sized up to the crate's historical repro constant (200,000): this render is `O(depth²)`
/// (each `Node::Let`/`Node::App`/… arm's `format!` copies its already-rendered sub-`String` into a
/// new buffer one level up — the same re-walk the shared budget's `WorkSteps` charge exists to
/// guard, per `mycelium-workstack`'s docs), so 200,000 levels pushes a **debug** build's per-frame
/// stack usage past the 256 MiB guard *even wrapped* (empirically confirmed) and the runtime into
/// tens of seconds; 20,000 keeps the census fast and comfortably within budget while still being
/// unambiguously deeper than anything the pre-fix code could survive.
#[test]
fn render_node_deep_let_chain() {
    let budget = RecursionBudget::with_depth_default(u64::MAX, u64::MAX);
    ensure_sufficient_stack(&budget, || {
        let n = 20_000;
        let deep = deep_let(n);
        let rendered = llm_canonical(&deep);

        // Clean completion: the process did not abort, and the render is well-formed — one
        // `(let [xI …` opener per nesting level (`deep_let` nests right-to-left, so the outermost
        // binder is `x{n-1}` and the innermost is `x0`, wrapping the original constant), plus the
        // constant payload.
        assert_eq!(rendered.matches("(let [x").count(), n);
        assert!(rendered.starts_with(&format!("(let [x{} ", n - 1)));
        assert!(rendered.contains("(let [x0 "));
        assert!(rendered.contains("(const 0b00000000"));
    });
}
