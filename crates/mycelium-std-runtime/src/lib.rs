//! `std.runtime` — the fungal concurrency surface (M-521 / ADR-020).
//!
//! Implements the v0 R1 API surface decided in ADR-020 (Accepted 2026-06-20):
//! [`Colony`]/[`Scope`] structured concurrency, [`Task`]/[`TaskCtx`]/[`Poll`],
//! sweep ordering ([`SweepOrder`], [`Deadlock`]), and the channel surface
//! ([`Network`], [`Sender`], [`Receiver`], [`TrySend`], [`TryRecv`]).
//!
//! # Guarantee matrix
//!
//! Every exported operation has a row in [`guarantee_matrix::MATRIX`].
//! The matrix is tested, not prose-only (SC-2 / VR-5).
//!
//! # Reserved vocabulary (Glossary ⟂)
//!
//! The RFC-0008 §4.5 vocabulary (`hypha`, `fuse`, `xloc`, `cyst`, `graft`,
//! `forage`, `backbone`, `mesh`, `tier`, `reclaim`) is **reserved but not
//! activated** in v0. None of these appear in this crate's public API.
//!
//! # `wild`-free
//!
//! This crate is `wild`-free in v0: no raw pointer transmutes, no `unsafe`
//! blocks, no `wild` keyword constructs (ADR-014).
//!
//! Design: `docs/adr/ADR-020-Runtime-Colony-Phylum-Placement.md`;
//! spec: `docs/spec/stdlib/runtime.md`; task M-521.
#![forbid(unsafe_code)]

pub mod colony;
pub mod guarantee_matrix;
pub mod network;
pub mod task;
