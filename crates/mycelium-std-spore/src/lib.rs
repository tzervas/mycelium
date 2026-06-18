//! `std.spore` — the deployable / reconstruction-manifest library (M-522, issue #163).
//!
//! `std.spore` is the ergonomic, value-semantic library face of the ADR-013 content-addressed
//! deployable unit and the RFC-0003 §6 reconstruction manifest. It **consumes** the landed
//! `mycelium-spore` packager (M-368), `std.content`'s hash primitives, and `std.vsa`'s decode;
//! it mints no new hash and performs no reconstruction itself (KC-3).
//!
//! # Honesty crux
//!
//! - **C4 / ADR-003:** a spore's identity *is* its canonical content hash; metadata is **not**
//!   identity, so the build hash is deterministic and metadata-invariant.
//! - **FR-C2 / VR-5 ceiling:** probabilistic VSA regrowth is tagged **`Empirical` at most**,
//!   carries its δ, and is **never** `Proven`. Enforced structurally: any manifest whose decode
//!   is `Resonator` and whose bound basis exceeds `Empirical` returns
//!   `Err(MalformedManifest::ResonatorOverStrength)` from [`ReconManifest::validate`].
//! - **C1 / G2:** a hash mismatch or missing/ambiguous publish input is an **explicit `Err`
//!   naming the offending input** — never a silent accept and never a partial artifact.
//!
//! # Guarantee matrix
//!
//! Every exported op has a row in [`guarantee_matrix::MATRIX`] (RFC-0016 §4.5 / spec §4).
//! The matrix is asserted in tests, not prose-only (C2 / VR-5).
//!
//! # Out of scope (FLAGS carried from spec §7)
//!
//! - **Native deploy / germination** (FLAG Q2) — Phase-6-gated (M-620); the `deploy` seam
//!   exists in the spec but is not implemented here — FLAGGED §7 Q2.
//! - **Approx<Value> coupling** (FLAG Q4) — `regrow` returns the raw `Factorization` from
//!   `std.vsa` rather than wrapping it in `std.numerics::Approx<Value>`. Coupling that type
//!   would add a workspace dependency on `mycelium-std-numerics` (a parallel-leaf crate).
//!   The orchestrator should wire the `Approx<Value>` wrapper after the wave merges. Until
//!   then [`RegrowthResult`] carries `(Factorization, GuaranteeStrength)` — inspectable and
//!   honest, never stronger than `Empirical`, but not yet the `Approx<Value>` envelope —
//!   FLAGGED §7 Q4.
//! - **Ring placement discrepancy** (FLAG Q1) — RFC-0016 §4.2 lists `spore` under Ring 2 while
//!   §4.3 / the stdlib index files it under Tier A. This spec follows Tier-A / Ring-1 —
//!   FLAGGED §7 Q1.
//! - **Packaging schema fields** (FLAG Q3) — the `mycelium-proj.toml` field set is
//!   M-368/M-359's to define; this crate consumes whatever they fix — FLAGGED §7 Q3.
//! - **Spore ↔ vsa seam** (FLAG Q4) — `std.vsa` owns the regrowth op; `spore` only packages
//!   and validates the manifest — co-design pending — FLAGGED §7 Q4.
//!
//! Design spec: `docs/spec/stdlib/spore.md`; ADR-013; RFC-0003 §6; task M-522, issue #163.
#![forbid(unsafe_code)]

pub mod guarantee_matrix;
pub mod recon_manifest;
pub mod spore_ops;

// Re-export the key consumer surface.
pub use mycelium_core::{ContentHash, GuaranteeStrength};
pub use mycelium_spore::{Spore as RawSpore, SporeError};

pub use guarantee_matrix::MATRIX;
pub use recon_manifest::{MalformedManifest, ReconManifest, ReconMode, RegrowthResult};
pub use spore_ops::{explain_spore, identity, manifest_of, verify, SporeErr, SporeUnit};
