//! `std.recover` — the declarative recovery bridge (M-520, issue #156; **Rust-first half only**).
//!
//! # Scope (honesty)
//!
//! This crate is the **Rust-first library surface** of `std.recover`. The **self-hosting migration**
//! half of M-520 (diag + recover authored in Mycelium-lang) is **Batch P5-C, M-502-gated** and is
//! *not* in this wave (`docs/spec/stdlib/recover.md` §Status).
//!
//! Recovery must be **additive / opt-in (I5)**, **never fabricate or upgrade a guarantee (I2)**,
//! **bounded (I3/I4)**, and **never a silent drop (I1)** — and it elaborates to an L0 `Match` with
//! **no new kernel node** (KC-3 / NFR-7). The recovered tag is **inherited from the policy, never
//! laundered upward**.
//!
//! Design spec: `docs/spec/stdlib/recover.md`; RFC-0014; task M-520, issue #156.
//!
//! ## Scaffold status (SCAFFOLD — M-520 leaf to complete)
//!
//! Stub surface only: the [`Outcome`] sum (no `Dropped` variant — I1) and the [`recover`] entry
//! point so the workspace builds. The M-520 leaf agent fills in: reified policy execution
//! (`run_policy`) with declared + bounded effects (`EffectBudgetExhausted` on overrun — I3/I4),
//! the recovered-tag ≤ policy-tag meet (I2 / VR-5 — fixes the P5-B exact-tag bug), the
//! [`mycelium_diag::Diag`] a recovered/re-propagated error carries (FR-R5), the §4.5 matrix, and the
//! never-drops / tag-never-upgraded / budget-bounded property tests. It branches from the
//! [`mycelium_diag`] scaffold contract — any change it needs to those record types is FLAGged to the
//! orchestrator, not edited here.
#![forbid(unsafe_code)]

use mycelium_core::{GuaranteeStrength, PolicyRef};

/// The result of a recovery attempt (FR-R1). There is **no `Dropped` variant** (I1): every error is
/// either recovered (with an honest, policy-inherited tag) or re-propagated.
#[derive(Debug, Clone, PartialEq)]
pub enum Outcome<T, E> {
    /// The error was recovered. `tag` is **≤ the policy's declared tag** (meet; never `Exact`
    /// unless the policy proves it — I2/VR-5). `policy` is the reified, EXPLAIN-able artifact.
    Recovered {
        /// The recovered value.
        value: T,
        /// The honest, policy-inherited guarantee tag.
        tag: GuaranteeStrength,
        /// The content-addressed policy that recovered this value.
        policy: PolicyRef,
    },
    /// The error was re-propagated (possibly transformed) — never silently dropped.
    Propagated(E),
}

/// Bridge a `Result` into an [`Outcome`] under a reified recovery `policy`.
///
/// SCAFFOLD: the `Ok` path stamps the honest **`Declared` floor** (never `Exact` — I2), and the
/// `Err` path re-propagates (the safe never-drop default). The M-520 leaf agent replaces the `Err`
/// arm with `run_policy` (declared + bounded effects, the closed action set), the recovered tag
/// being `action_tag.meet(policy_tag)`.
#[must_use]
pub fn recover<T, E>(r: Result<T, E>, policy: PolicyRef) -> Outcome<T, E> {
    match r {
        Ok(value) => Outcome::Recovered {
            value,
            tag: GuaranteeStrength::Declared,
            policy,
        },
        // Never a drop (I1): until a policy action is run, the honest behaviour is re-propagation.
        Err(e) => Outcome::Propagated(e),
    }
}
