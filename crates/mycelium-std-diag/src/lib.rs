//! `std.diag` — the structured failure-legibility surface (M-510, issue #151).
//!
//! `std.diag` is the ergonomic library face of the RFC-0013 structured-diagnostic record. The
//! canonical record types (`Diag`, `Severity`, `Locus`, `Trace`, `Code`) live in the
//! [`mycelium_diag`] kernel crate (the maintainer-resolved homing decision — see that crate's docs);
//! this module **re-exports** them and adds the ergonomic surface (the dual human/JSON projection,
//! builders, the §4.5 guarantee matrix). KC-3: it adds no trusted record algebra — it consumes the
//! kernel record.
//!
//! # Honesty crux (RFC-0013 I1)
//!
//! Presentation **never gates propagation**: a `Diag` is additive over an already-explicit error
//! and structurally incapable of replacing it. `Diag::human()` / `Diag::machine()` are dual
//! projections (G11); a `Diag` survives recover/re-propagate with its identity unchanged (ADR-003).
//!
//! Design spec: `docs/spec/stdlib/diag.md`; RFC-0013; task M-510, issue #151.
//!
//! ## Scaffold status (SCAFFOLD — M-510 leaf to complete)
//!
//! Stub surface only: re-exports the kernel record so the workspace builds. The M-510 leaf agent
//! fills in: `human`/`machine` dual projections (delegating canonical JSON to `std.io` is FLAGGED —
//! spec §7-Q1), content-addressed identity, the §4.5 guarantee matrix as checked data, and the
//! severity-ordering / never-silent / round-trip property tests. **Coordinate:** reconcile
//! `mycelium-std-testing`'s placeholder `FailRecord` to delegate to this `Diag` (FLAG to orchestrator).
#![forbid(unsafe_code)]

pub use mycelium_diag::{Code, Diag, Locus, Severity, Trace};
