//! Run a bounded-but-deep recursive pass on a worker thread with a large, explicit stack.
//!
//! The L1 frontend's **checker** ([`crate::checkty`]) and **elaborator** ([`crate::elab`]) recurse
//! over the expression AST. Three honesty/robustness facts shape how deep that recursion may go:
//!
//! - The **parser** bounds surface nesting at [`crate::parse::MAX_EXPR_DEPTH`] (a clean refusal — A4-02).
//! - The **checker** carries its own explicit budget (`checkty::MAX_CHECK_DEPTH`), and the
//!   **evaluator** its own (`eval::DEFAULT_DEPTH`) — each a *semantic* ceiling that refuses with an
//!   explicit error, never a crash (the "banked guard 4" discipline).
//! - But correctness must **not** silently depend on the *caller's* thread-stack size. A worker
//!   thread is commonly 2 MiB; the `main` stack is larger; either way the host-stack budget is a
//!   resource, not a semantic limit — and it changes as the IR grows (e.g. a wider `Ty`).
//!
//! So we run the recursive pass on a dedicated worker thread with a large, **lazily-committed** stack:
//! the address space is *reserved* up front (cheap — no physical pages), and pages are touched only
//! as recursion actually deepens, so a shallow program pays ~nothing. This is the std-only,
//! `unsafe`-free counterpart to a `stacker`-style segmented stack — it keeps the trusted base
//! **dependency-free** while making deep input robust on *any* caller thread.
//!
//! Honesty: this raises the *host-stack* ceiling; it is **not** a semantic budget. The semantic
//! ceilings stay explicit and never-silent (parser / checker / evaluator each refuse past their
//! budget with a clean error). The worker stack is sized to comfortably exceed what those budgets
//! admit, so a pathological input is always bounded by an explicit budget — never by a stack overflow.
//!
//! **Self-hosting note (the load-bearing distinction).** The L1 frontend is slated to be rewritten in
//! Mycelium itself, whose native execution model is *value-semantic, immutable/acyclic, and
//! fuel/clock-bounded* (RFC-0007 §4.5/§4.6; RFC-0008 runtime) — it has **no host call stack to grow**.
//! So the portable, first-class primitive is the **explicit, reified depth budget** (`MAX_EXPR_DEPTH`,
//! `MAX_CHECK_DEPTH`, the evaluator's clock) — those carry over *directly* to the self-hosted version,
//! which will trade unbounded host recursion for an explicit heap work-stack walked under a budget
//! (the same shape as the evaluator's clocked big-step machine). **This module is the transitional
//! Rust-only adapter**: it lets the bounded recursion fit the host stack *while the frontend is still
//! Rust*, and is expected to disappear when the frontend self-hosts (the budgets stay; the stack
//! sizing does not). Keep the budgets first-class; treat the stack sizing as scaffolding.

/// A generous worker-thread stack for the recursive trusted passes. Reserved virtually; committed
/// lazily, so a shallow pass touches only a handful of pages. Comfortably exceeds what the explicit
/// depth budgets (`parse::MAX_EXPR_DEPTH`, `checkty::MAX_CHECK_DEPTH`) admit at any plausible frame
/// size, so those budgets — not a host-stack overflow — always bound a pathological input.
const DEEP_STACK_BYTES: usize = 256 * 1024 * 1024;

/// Run `f` on a worker thread with a large explicit stack ([`DEEP_STACK_BYTES`]) and return its
/// value. A panic inside `f` is propagated to the caller unchanged (via [`std::panic::resume_unwind`]),
/// so assertions and `#[should_panic]` behave exactly as if `f` had run inline. The closure may borrow
/// from the caller (scoped thread), so the recursive passes keep taking `&Nodule` / `&Env` by reference.
pub(crate) fn with_deep_stack<T, F>(f: F) -> T
where
    F: FnOnce() -> T + Send,
    T: Send,
{
    std::thread::scope(|scope| {
        std::thread::Builder::new()
            .name("mycelium-l1-deep".to_owned())
            .stack_size(DEEP_STACK_BYTES)
            .spawn_scoped(scope, f)
            .expect("spawn the deep-stack worker thread")
            .join()
            .unwrap_or_else(|panic| std::panic::resume_unwind(panic))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runs_the_closure_and_returns_its_value() {
        assert_eq!(with_deep_stack(|| 2 + 2), 4);
    }

    #[test]
    fn the_closure_may_borrow_from_the_caller() {
        let xs = [1u64, 2, 3];
        let sum: u64 = with_deep_stack(|| xs.iter().sum());
        assert_eq!(sum, 6);
    }

    #[test]
    fn a_genuinely_deep_recursion_does_not_overflow() {
        // Far past any default thread stack (2 MiB) at a non-trivial frame size — the lazily-committed
        // worker stack absorbs it without a crash. This is the regression guard for the "deep input
        // must never overflow the caller's stack" contract.
        fn descend(n: u64, pad: &[u8; 512]) -> u64 {
            if n == 0 {
                pad[0] as u64
            } else {
                // `pad` keeps each frame ~non-trivial, so this really exercises stack depth.
                descend(n - 1, pad).wrapping_add(1)
            }
        }
        let got = with_deep_stack(|| descend(200_000, &[7u8; 512]));
        assert_eq!(got, 200_007);
    }

    #[test]
    #[should_panic(expected = "intentional")]
    fn a_panic_in_the_closure_propagates_to_the_caller() {
        with_deep_stack(|| panic!("intentional"));
    }
}
