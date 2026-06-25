//! `std.runtime` — the fungal concurrency surface (M-521 / ADR-020).
//!
//! Implements the v0 R1 API surface decided in ADR-020 (Accepted 2026-06-20):
//! [`colony::Colony`]/[`colony::Scope`] structured concurrency,
//! [`task::Task`]/[`task::TaskCtx`]/[`task::Poll`],
//! sweep ordering ([`task::SweepOrder`], [`task::Deadlock`]), and the channel surface
//! ([`network::Network`], [`network::Sender`], [`network::Receiver`], [`network::TrySend`], [`network::TryRecv`]).
//!
//! # Guarantee matrix
//!
//! Every exported operation has a row in [`guarantee_matrix::MATRIX`].
//! The matrix is tested, not prose-only (SC-2 / VR-5).
//!
//! # Execution maturity (E12-1: M-709 / M-711 / M-713)
//!
//! Beyond the cooperative v0 surface, the crate now executes on real OS threads:
//! - [`scheduler::Scheduler`] (M-709) — a fixed OS-thread pool with fair FIFO dispatch and
//!   demand-signalled, bounded backpressure (RFC-0008 RT1·RT2·§4.3); the RT2 sequentialization
//!   differential is property-tested (`Empirical`).
//! - [`dataflow::run_dataflow`] / [`dataflow::run_dataflow_scheduled`] (M-711) — deadlock-freedom
//!   for communicating tasks: a no-progress sweep is an explicit [`task::Deadlock`], never a silent
//!   hang (G2 / RFC-0008 §4.3), checked on both the cooperative path and the OS-thread pool.
//! - [`supervision`] (M-713) — structured-concurrency cancellation ([`supervision::CancelTree`]),
//!   explicit per-child outcome collection ([`supervision::run_supervised`]), and an EXPLAIN-able
//!   bounded-cascade restart policy ([`supervision::supervise_with_restart`]) (RFC-0008 RT4·RT7),
//!   reusing the M-356 composition kernel from `mycelium-interp`.
//!
//! # Reserved vocabulary (Glossary ⟂)
//!
//! The RFC-0008 §4.5 surface **constructs** (`hypha`, `fuse`, `xloc`, `cyst`, `graft`,
//! `forage`, `backbone`, `mesh`, `tier`, `reclaim`) remain **reserved, not yet activated** as
//! L1 language surface (their elaboration is M-710, gated on the M-667 L1 surface). The runtime
//! *machinery* they will dispatch to (scheduler, deadlock detection, supervision) is what this
//! crate now provides.
//!
//! # `wild`-free
//!
//! This crate is `wild`-free: no raw pointer transmutes, no `unsafe`
//! blocks, no `wild` keyword constructs (ADR-014).
//!
//! Design: `docs/adr/ADR-020-Runtime-Colony-Phylum-Placement.md`;
//! spec: `docs/spec/stdlib/runtime.md`; tasks M-521, E12-1 (M-709/M-711/M-713).
//!
//! # Memory model (E12 MEM-1/MEM-2/MEM-3)
//!
//! - [`reclamation`] (MEM-1) — the reclamation EXPLAIN/audit record and never-silent sink
//!   contract (RFC-0027 §9).
//! - [`rc`] (MEM-2) — non-atomic intra-hypha RC cell + `rc`-probe decision (DN-32 §2.2).
//! - [`region`] (MEM-3) — region-based batched scope-exit reclamation (DN-32 §2.3 / RFC-0027
//!   §10.3): [`region::Region`] accumulates deferred entries and bulk-emits `ScopeExit` records
//!   at scope-exit; [`region::ScopeNodeId`] / [`region::RegionEpoch`] are the canonical forms
//!   of the MEM-1 `ScopeId`/`SweepEpoch` placeholder types.
#![forbid(unsafe_code)]

pub mod colony;
pub mod dataflow;
pub mod guarantee_matrix;
pub mod network;
pub mod rc;
pub mod reclamation;
pub mod region;
pub mod scheduler;
pub mod supervision;
pub mod task;

#[cfg(test)]
mod tests;
