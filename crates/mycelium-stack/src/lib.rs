//! **Host-stack management for the L1 frontend's recursive passes — isolated outside the trusted
//! kernel.**
//!
//! The L1 checker ([`mycelium_l1::checkty`]) and elaborator ([`mycelium_l1::elab`]) recurse over the
//! expression AST. They must never overflow the *caller's* thread stack on deep input — but the
//! caller's stack size is a host resource, not a semantic limit, and it varies (a 2 MiB worker thread
//! vs the larger `main` stack), and it interacts with frame size (which grows as the IR evolves).
//! Coupling correctness to that resource is fragile. This crate breaks the coupling **without putting
//! any `unsafe` in the trusted kernel**.
//!
//! ## Why a separate crate (the architecture)
//!
//! - **The kernel stays `unsafe`-free and *machine-proven*.** `mycelium-l1` is `#![forbid(unsafe_code)]`
//!   (ADR-014: "trusted-base crates should stay unsafe-free; may re-pin `unsafe_code = forbid`
//!   per-crate"). All host-stack machinery lives **here**, behind a safe API the kernel calls.
//! - **The semantic budgets stay in the kernel.** The parser caps surface nesting
//!   (`mycelium_l1::parse::MAX_EXPR_DEPTH`), the checker carries `MAX_CHECK_DEPTH`, and the evaluator a
//!   per-node depth clock — each an **explicit, reified budget** that refuses past it with a clean
//!   error, never a crash (the "banked guard 4" discipline). *Those* are the bound on a pathological
//!   input. This crate only ensures the host stack is large enough that the **budget**, not an
//!   overflow, is always what stops it.
//! - **Self-hosting:** the explicit budgets are the **portable primitive** — they carry directly to the
//!   future Mycelium-native frontend, whose value-semantic, fuel/clock-bounded model has *no host call
//!   stack to grow* (RFC-0007 §4.5/§4.6). **This crate is the transitional Rust-host adapter** and is
//!   expected to disappear when the frontend self-hosts (the budgets stay; the stack sizing does not).
//!
//! ## The default: a deep, lazily-committed worker stack (`unsafe`-free)
//!
//! [`with_deep_stack`] runs the recursive pass on a dedicated worker thread with a large explicit
//! stack. The address space is *reserved* up front (cheap) and physical pages are touched only as
//! recursion actually deepens — so a shallow program pays ~nothing (the same "pay for the depth you
//! use" benefit as a segmented stack), with **zero `unsafe`**: it is pure `std::thread`.
//!
//! ## The hybrid extension (documented, contained — never in the kernel)
//!
//! A "large stack **+** grow-on-demand" hybrid (so recursion can grow *past* the reserved stack) is a
//! one-step addition **in this crate**: an optional `grow-on-demand` feature wrapping
//! `stacker::maybe_grow` (a *safe* API — the stack-switching `unsafe` is internal to the `stacker`
//! leaf crate). Even then this crate's own source stays `unsafe`-free, so the `unsafe` is shunted to a
//! single audited upstream leaf, the furthest possible point from the kernel. It is **off by default**;
//! the std-only path is the robust, dependency-free baseline (see `Cargo.toml`).
#![forbid(unsafe_code)]

/// A generous worker-thread stack for the recursive trusted passes. Reserved virtually; committed
/// lazily, so a shallow pass touches only a handful of pages. Comfortably exceeds what the kernel's
/// explicit depth budgets admit at any plausible frame size, so those budgets — not a host-stack
/// overflow — always bound a pathological input. (Measured: the L1 checker uses ~10.9 KiB/frame in
/// debug, so 256 MiB physically supports ~24,600 levels — ~6× the checker's 4096 budget.)
const DEEP_STACK_BYTES: usize = 256 * 1024 * 1024;

/// Run `f` on a worker thread with a large explicit stack ([`DEEP_STACK_BYTES`]) and return its value.
///
/// A panic inside `f` is propagated to the caller unchanged (via [`std::panic::resume_unwind`]), so
/// assertions and `#[should_panic]` behave exactly as if `f` had run inline. The closure may borrow
/// from the caller (it runs on a *scoped* thread), so the recursive passes keep taking `&Nodule` /
/// `&Env` by reference. `unsafe`-free: pure `std::thread`.
///
/// Cost: one worker-thread spawn per call (tens of microseconds) — negligible for a compiler pass and
/// paid once at the top of `check`/`elaborate`, not per recursion level. The large stack is virtual
/// address space, committed lazily, so a shallow pass uses only a few pages.
pub fn with_deep_stack<T, F>(f: F) -> T
where
    F: FnOnce() -> T + Send,
    T: Send,
{
    std::thread::scope(|scope| {
        std::thread::Builder::new()
            .name("mycelium-deep-stack".to_owned())
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
