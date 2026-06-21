//! `adk` — a Rust-first port of Google's Agent Development Kit, per **RFC-0023** (M-671).
//!
//! # The differentiator: honesty (RFC-0023 §6)
//! The Mycelium port is **honest where Python ADK is silent** — and this holds *regardless of
//! model quality*, because the substrate forbids the dishonest move structurally:
//! - an LLM output is **type-forbidden from `Proven`/`Exact`** (`model_allowed_tags =
//!   {Declared, Empirical}`; the tag is preserved verbatim, **never upgraded**),
//! - a tool failure is an explicit `Err(ToolError)` — never a silent `None`/default (C1/G2),
//!   and a synthetic/mock run is flagged ([`model::is_synthetic`]), never reported as real.
//!
//! # Status — scaffold + pure data model, pending ratification
//! RFC-0023 is **`Draft`**; the `dfr` deep-research pass (**RP-9**) gates its Honest-Uncertainty
//! Register. This session builds the **non-gated pure data model + honesty discipline** only.
//! FLAGGED-gated (never faked):
//! - [`runner`]'s colony-driven loop — pending the runtime-realization choice (R23-Q1:
//!   `mycelium-std-runtime::colony` vs `mycelium-mlir::runtime`),
//! - [`model`]'s real `generate` call — real LLM runs are excluded from this wave (M-381/M-646),
//! - the typed `Tool<In,Out>` / `Agent` and `colony { … }` `.myc` surfaces (E7-1 / E7-2).
//!
//! # Nodules (RFC-0023 §4)
//! [`tool`] · [`agent`] · [`session`] · [`runner`] · [`model`].
#![forbid(unsafe_code)]

pub mod agent;
pub mod guarantee_matrix;
pub mod model;
pub mod runner;
pub mod session;
pub mod tool;
