//! `adk.runner` — the colony scheduler stub (RFC-0023 §4.4).
//!
//! The runner drives the agent↔tool↔model event loop.  The **colony-driven loop** is
//! **FLAGGED-deferred** on R23-Q1 (the runtime-realization choice — RFC-0023 §7.1 /
//! §10 R23-Q1): two realizations exist (`mycelium-std-runtime::colony::Scope::join_all`
//! and `mycelium-mlir::runtime`) and the port must pick one deliberately.  Until R23-Q1
//! is resolved, `run` returns an explicit `Err` — **never a fabricated** `(Session, events)`.
//!
//! ## Types
//! - [`RunError`] — `AgentFailed(String) | BudgetExhausted | Cancelled`
//!
//! ## Deferred (FLAG R23-Q1)
//! - `run(agent, session, message) -> Result<(Session, Vec<Event>), RunError>` signature
//!   exists; the body returns `Err(RunError::AgentFailed("runner gated on R23-Q1 …"))`.
//!
//! ## Honesty (VR-5 / C1)
//! The stub MUST return explicit `Err` — **not** `Ok` with fabricated events.  Any
//! change that makes `run` return `Ok(...)` for any input is a honesty violation until
//! R23-Q1 is resolved and a real colony-driven loop is implemented.
//!
//! ## Design spec
//! `docs/rfcs/RFC-0023-Agent-Development-Kit-Phylum.md` §4.4, §7.1, §10 R23-Q1

use std::fmt;

use crate::agent::Agent;
use crate::session::{Event, Session};

// ── RunError ──────────────────────────────────────────────────────────────────

/// The explicit error set for runner failures (RFC-0023 §4.4).
///
/// Mirrors `TaskOutcome::Failed`/`BudgetExhausted`/`Cancelled` from
/// `crates/mycelium-interp/src/supervise.rs:94` (`enum TaskOutcome`).
///
/// # Honesty (C1 / VR-5)
/// No variant means "proceed as if the agent succeeded."  A failed run is an explicit
/// `Err(RunError)` the caller must handle.  In particular, `AgentFailed` carries the
/// reason string — never an empty/silent failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunError {
    /// The agent's execution failed.  `0` carries a human-readable reason (G11).
    ///
    /// This is also the variant returned by the R23-Q1 stub — with a reason string
    /// that names the unresolved gate, so the caller can distinguish a stub failure
    /// from a real runtime failure.
    AgentFailed(String),

    /// The runner's declared spend/compute budget was exhausted.
    ///
    /// The run stops with a partial, honestly-flagged report — never a silent truncation
    /// (C6 / RFC-0023 §5 — "the run stops with a partial, honestly-flagged report").
    BudgetExhausted,

    /// The run was cancelled (externally or by an `escalate` signal from a sub-agent).
    Cancelled,
}

impl fmt::Display for RunError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RunError::AgentFailed(reason) => write!(f, "agent failed: {reason}"),
            RunError::BudgetExhausted => write!(f, "runner budget exhausted"),
            RunError::Cancelled => write!(f, "run cancelled"),
        }
    }
}

mycelium_std_core::impl_std_error!(RunError);

// ── run (gated stub) ──────────────────────────────────────────────────────────

/// Run an agent against a session with an initial message.
///
/// # Current behaviour — DEFERRED (FLAG R23-Q1)
/// Returns `Err(RunError::AgentFailed("runner gated on R23-Q1 …"))` always.
///
/// The colony-driven loop (RFC-0023 §4.4) is **gated on R23-Q1** — the runtime-
/// realization choice between `mycelium-std-runtime::colony::Scope::join_all` and
/// `mycelium-mlir::runtime` has not been resolved (RFC-0023 §7.1 / §10 R23-Q1).
/// The port must pick **one** deliberately and justify it; guessing is a VR-5 violation.
///
/// # Honesty contract (VR-5 / C1)
/// This function MUST return `Err` until R23-Q1 is resolved and a real loop is
/// implemented.  Any implementation that returns `Ok(...)` with fabricated events
/// is a **honesty violation** — it would present a mock run as a real one (RFC-0023
/// §6.5 / §7.4 — "the spec moves 'implemented (Rust-first), pending ratification',
/// never silently to Accepted").
///
/// # Future signature (RFC-0023 §4.4 TARGET — Mycelium language)
/// ```mycelium
/// fn run(agent: Agent, session: Session, message: Content)
///     -> Result<(Session, List<Event>), RunError> =
///   colony {
///     hypha step(agent, session, message)
///   }
/// ```
/// The `colony`/`hypha` constructs need E7-2/M-666.
///
/// # FLAG (E7-2 / M-666)
/// The Mycelium-language `colony { hypha … }` surface is deferred (parser construct
/// M-666 not yet active; lexer reservation has landed — M-665).
pub fn run(
    _agent: &Agent,
    _session: &Session,
    _message: &str,
) -> Result<(Session, Vec<Event>), RunError> {
    // FLAG R23-Q1: colony-driven loop is deferred.
    // Two runtime realizations exist; the port must pick one deliberately.
    // Returning Err is the honest stance — never a fabricated (Session, events).
    Err(RunError::AgentFailed(
        "runner gated on R23-Q1: runtime realization choice unresolved \
         (mycelium-std-runtime::colony::Scope::join_all vs mycelium-mlir::runtime — \
         RFC-0023 §7.1 / §10 R23-Q1)"
            .to_owned(),
    ))
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::{run, RunError};
    use crate::agent::{Agent, Instruction, LlmAgent, ModelRef};
    use crate::session::Session;
    use proptest::prelude::*;

    fn test_agent() -> Agent {
        Agent::Llm(LlmAgent {
            name: "test".to_owned(),
            model: ModelRef::new("grok"),
            instruction: Instruction::Static("be helpful".to_owned()),
            description: "test agent".to_owned(),
            tools: vec![],
            sub_agents: vec![],
            output_key: None,
        })
    }

    // ── Unit tests ─────────────────────────────────────────────────────────────

    /// `run` returns `Err(AgentFailed(...))` while R23-Q1 is unresolved — never `Ok`.
    /// Guard: any change that makes `run` return `Ok` is a honesty violation (VR-5).
    #[test]
    fn run_returns_err_agent_failed_while_gated_never_fabricates_ok() {
        let agent = test_agent();
        let session = Session::new();
        let result = run(&agent, &session, "hello");
        assert!(
            matches!(result, Err(RunError::AgentFailed(_))),
            "run must return Err(AgentFailed) while R23-Q1 is unresolved; \
             an Ok result would be a fabricated success (VR-5); got {result:?}"
        );
    }

    /// The `AgentFailed` reason string mentions R23-Q1 (for debuggability — G11).
    #[test]
    fn run_err_reason_mentions_r23_q1() {
        let agent = test_agent();
        let session = Session::new();
        let Err(RunError::AgentFailed(reason)) = run(&agent, &session, "msg") else {
            panic!("expected AgentFailed");
        };
        assert!(
            reason.contains("R23-Q1"),
            "the AgentFailed reason must name the unresolved gate (R23-Q1) for debuggability (G11); got {reason:?}"
        );
    }

    /// `RunError` implements `std::error::Error` (compile-time check).
    #[test]
    fn run_error_is_std_error() {
        let e = RunError::AgentFailed("x".to_owned());
        let _: &dyn std::error::Error = &e;
    }

    /// `RunError` Display is non-empty for every variant (G11 — human-legible).
    #[test]
    fn run_error_display_non_empty_for_every_variant() {
        let variants = [
            RunError::AgentFailed("reason".to_owned()),
            RunError::BudgetExhausted,
            RunError::Cancelled,
        ];
        for v in &variants {
            let s = v.to_string();
            assert!(
                !s.is_empty(),
                "RunError::{v:?} must have non-empty Display (G11)"
            );
        }
    }

    /// `RunError::BudgetExhausted` Display contains "budget" (for legibility).
    #[test]
    fn run_error_budget_exhausted_display_contains_budget() {
        let s = RunError::BudgetExhausted.to_string();
        assert!(
            s.contains("budget"),
            "BudgetExhausted Display must mention 'budget'"
        );
    }

    // ── Property tests ────────────────────────────────────────────────────────

    proptest! {
        /// BOUND: `run` is ALWAYS `Err` for any input while R23-Q1 is unresolved.
        /// Guard: any implementation that returns `Ok` for any input breaks this.
        #[test]
        fn prop_run_is_always_err_while_gated(message in "[a-z ]{1,50}") {
            let agent = test_agent();
            let session = Session::new();
            let result = run(&agent, &session, &message);
            prop_assert!(
                result.is_err(),
                "run must always return Err while R23-Q1 is unresolved (VR-5); \
                 got Ok for message {message:?}"
            );
        }

        /// BOUND: `RunError::AgentFailed` Display always contains the provided reason.
        #[test]
        fn prop_agent_failed_display_includes_reason(reason in "[a-z]{1,30}") {
            let e = RunError::AgentFailed(reason.clone());
            prop_assert!(
                e.to_string().contains(&reason),
                "AgentFailed Display must include the reason (G11); got {:?}", e.to_string()
            );
        }
    }
}
