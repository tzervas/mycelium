//! `std.vsa` (`hdc`) — Ring 1 / Tier A ergonomic capability surface over the landed VSA/HDC
//! models (M-513; RFC-0016 §4.2/§4.3).
//!
//! # Design contract (RFC-0016 §4.1, C1–C6)
//!
//! - **C1 — never-silent:** every fallible op returns an explicit `Result`; out-of-capacity,
//!   below-threshold, ambiguous, mismatched-model/dim, and empty-bundle are explicit errors,
//!   never sentinel values or silent coercions (G2).
//! - **C2 — honest per-op tag:** each `(model, op)` row carries the tag the RFC-0003 §4 matrix
//!   assigns — read from `mycelium_vsa::matrix_tag`, never fabricated. The guarantee matrix is
//!   encoded as data ([`GUARANTEE_MATRIX`]) and asserted in tests, not prose only (RFC-0016 §4.5).
//! - **C3 — no black boxes / EXPLAIN:** every selecting/approximating op exposes *why* — cleanup
//!   returns `(label, confidence, margin)`; a resonator run returns a `ResonatorTrace` (SC-3/G11).
//! - **C4 — content-addressed, value-semantic:** every op is a pure function of its inputs (C4).
//! - **C5 — above the small kernel:** no new trusted code; wraps `mycelium-vsa` (KC-3).
//! - **C6 — declared, bounded effects:** every op is pure; resonator iteration is bounded (C6).
//!
//! # Guarantee matrix (RFC-0016 §4.5)
//!
//! [`GUARANTEE_MATRIX`] is the load-bearing data table: one [`OpGuarantee`] row per `(model, op)`
//! pair, mirroring the normative RFC-0003 §4 matrix as corrected by the r3 §4.1 erratum and
//! encoded in `mycelium_vsa::matrix_tag`.  Tests assert the table against the kernel matrix so
//! divergence is caught mechanically (VR-5).
//!
//! # Scope / boundary (vsa.md §2)
//!
//! Out of scope here: dense tensors (`std.dense` M-518), content-addressing (`std.content`
//! M-523), deployable spore (`std.spore` M-522), `Dense ↔ VSA` repr change (`std.swap` M-516),
//! ε/δ bound kernels (`std.numerics` M-512).  No new `unsafe`; no new `Repr` kind; no new model.

#![forbid(unsafe_code)]

pub mod encoding;
pub mod matrix;
pub mod ops;
pub mod recon;

// Re-export the key types consumers need without forcing them to depend on the kernel crates
// directly for the common VSA surface.
pub use mycelium_core::GuaranteeStrength;
pub use mycelium_vsa::{
    CleanupMemory, Factorization, Match, ResonatorTrace, VsaError, VsaModel, VsaOp,
};

pub use encoding::{encode_seq, encode_set};
pub use matrix::{OpGuarantee, GUARANTEE_MATRIX};
pub use ops::{bind, bind_role, bundle, cleanup, permute, similarity, unbind, unpermute};
pub use recon::{reconstruct_factors, reconstruct_role};
