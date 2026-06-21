//! `mycelium-adk` — Agent Development Kit phylum (RFC-0023, M-671).
//!
//! A Mycelium port of Google ADK: typed tools, pure-data agent definitions,
//! content-addressed session/state, a (gated) runner, and a model layer.
//! The **differentiator** (RFC-0023 §6) is honesty:
//!
//! - An LLM outcome's guarantee tag is **type-forbidden** from `Proven`/`Exact`
//!   (only `Declared`|`Empirical` are allowed — `model_allowed_tags`).
//! - Tool failures are **never silent** (`Result<_, ToolError>`, always `Err`).
//! - Deferred/gated paths return explicit `Err`, never a fabricated success.
//!
//! ## Status
//! **Rust-first**, data-model + honesty discipline only.  The runtime loop
//! (`runner`) is gated on R23-Q1 (runtime-realization choice).  Real LLM calls
//! are excluded this wave (M-381/M-646).  Mycelium-language surface (§4) targets
//! E7-1/E7-2 completion.
//!
//! ## Nodules (modules)
//! - [`tool`] — `ToolError`, `Tool` trait, pure tool surface (`adk.tool`)
//! - [`agent`] — pure data: `LlmAgent`, `Workflow`, `Agent`, id newtypes (`adk.agent`)
//! - [`session`] — `State`, `Event`, `Session`; value-semantic `put` (`adk.session`)
//! - [`model`] — `ModelError`, `LlmRequest`, `LlmOutcome`, honesty guard (`adk.model`)
//! - [`runner`] — `RunError`, `run` stub (gated, R23-Q1) (`adk.runner`)
//! - [`guarantee_matrix`] — per-op matrix + load-bearing LLM-tag guard tests
//!
//! ## Design spec
//! `docs/rfcs/RFC-0023-Agent-Development-Kit-Phylum.md`
#![forbid(unsafe_code)]

pub mod agent;
pub mod guarantee_matrix;
pub mod model;
pub mod runner;
pub mod session;
pub mod tool;
