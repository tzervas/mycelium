//! `std.diag` — the structured failure-legibility surface (M-510, issue #151).
//!
//! `std.diag` is the ergonomic library face of the RFC-0013 structured-diagnostic record. The
//! canonical record types (`Diag`, `Severity`, `Locus`, `Trace`, `Code`) live in the
//! [`mycelium_diag`] kernel crate (the maintainer-resolved homing decision — see that crate's
//! docs); this module **re-exports** them and adds the ergonomic surface and the §4.5 guarantee
//! matrix as checked data. KC-3: it adds no trusted record algebra — it consumes the kernel record.
//!
//! # Honesty crux (RFC-0013 I1)
//!
//! Presentation **never gates propagation**: a `Diag` is additive over an already-explicit error
//! and structurally incapable of replacing it. [`Diag::human`] / [`Diag::machine`] are dual
//! projections (G11); a `Diag` survives recover/re-propagate with its identity unchanged (ADR-003).
//!
//! # Guarantee matrix (RFC-0016 §4.5 — the load-bearing deliverable)
//!
//! The §4.5 matrix is encoded as **data** in the [`guarantee_matrix`] module and asserted in
//! tests — never prose-only. All `diag` ops are `Exact` (the module has no accuracy semantics of
//! its own; RFC-0016 C2 "len-style" case). The only place a lattice tag appears is as *reported*
//! data — `guarantee` / `audit_of` surface honest tags without upgrading them (RT5/VR-5).
//!
//! Design spec: `docs/spec/stdlib/diag.md`; RFC-0013; task M-510, issue #151.
//!
//! ## Ambient Representation (RFC-0012 §8-Q3)
//!
//! This crate's public API participates in the RFC-0012 ambient-representation contract:
//! the representation choice (binary/ternary/dense/VSA) is implicit at the call site but
//! always reified, queryable, and EXPLAIN-able — never a black box (C3/SC-3).
//! [Declared per RFC-0012; direction accepted in DN-07 §8-Q3; per-ring pass scheduled as M-540.]
//!
//! **For this crate (Ring 1, Tier A):** Diagnostic emissions include representation context —
//! a `Diag` record that originates from a representation operation carries the `Repr` of the
//! input and output as part of its `Locus`; EXPLAIN-over-representation is a first-class
//! diagnostic kind. `Diag::human` and `Diag::machine` are dual projections (G11); representation
//! context is present in both projections, never silently omitted.
#![forbid(unsafe_code)]

// Re-export the kernel record types so consumers only need to depend on this crate.
pub use mycelium_diag::{Code, ContentHash, Diag, Locus, Severity, Trace};

/// The §4.5 guarantee matrix — encoded as data, asserted in tests (RFC-0016 §4.5; spec §4).
pub mod guarantee_matrix;
