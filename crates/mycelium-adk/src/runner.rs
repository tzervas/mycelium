//! nodule: `adk.runner` — the colony scheduler driving the agent ↔ tool ↔ model loop,
//! per RFC-0023 §4 / §7.1.
//!
//! Build now: the `run(agent, session, message) -> Result<(Session, Vec<Event>), RunError>`
//! signature and `RunError { AgentFailed | BudgetExhausted | Cancelled }`.
//!
//! FLAGGED-gated (never faked): the **colony-driven drive loop** is deferred pending the
//! runtime-realization choice (R23-Q1: `mycelium-std-runtime::colony` vs `mycelium-mlir::runtime`)
//! — RFC-0023 §7.1 lists two and does not pick; RP-9 decides. The deferred `run` returns an
//! explicit, never-silent `Err`, not a fabricated result.
