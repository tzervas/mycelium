//! `std.spore` — the deployable / reconstruction-manifest library (M-522, issue #163).
//!
//! `std.spore` is the ergonomic, value-semantic library face of the ADR-013 content-addressed
//! deployable unit and the RFC-0003 §6 reconstruction manifest. It **consumes** the landed
//! `mycelium-spore` packager (M-368), `std.content`'s hash primitives, and `std.vsa`'s decode; it
//! mints no new hash and performs no reconstruction itself (KC-3).
//!
//! # Honesty crux
//!
//! - **C4 / ADR-003:** a spore's identity *is* its canonical content hash; metadata is **not**
//!   identity, so the build hash is deterministic and metadata-invariant.
//! - **FR-C2 / VR-5 ceiling:** probabilistic VSA regrowth is tagged **`Empirical` at most**, carries
//!   its δ, and is **never** `Proven`.
//! - **C1 / G2:** a hash mismatch or missing/ambiguous publish input is an **explicit `Err` naming
//!   the offending input** — never a silent accept and never a partial artifact.
//!
//! The full **native** deploy / germination is Phase-6-gated (M-620) and explicitly **out of scope**
//! here (FLAGGED — spec §7 Q2); this is the library / manifest half.
//!
//! Design spec: `docs/spec/stdlib/spore.md`; ADR-013; RFC-0003 §6; task M-522, issue #163.
//!
//! ## Scaffold status (SCAFFOLD — M-522 leaf to complete)
//!
//! Stub surface only: the [`Spore`] handle + [`SporeErr`] and a [`Spore::verify`] signature so the
//! workspace builds. The M-522 leaf agent fills in: `spore(v)` packaging over `mycelium-spore`, the
//! manifest read/verify (total/`Exact`; hash-mismatch → `Err`), VSA-backed `regrow` held at the
//! `Empirical` ceiling with its δ, the §4.5 matrix, and the round-trip / hash-mismatch-refusal /
//! regrowth-ceiling property tests. If `regrow` should return `std.numerics::Approx`, that adds a
//! workspace dependency — **FLAG to the orchestrator**, do not edit the workspace manifest.
#![forbid(unsafe_code)]

use mycelium_core::ContentHash;

/// An explicit spore error — never a silent accept (C1/G2).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SporeErr {
    /// The recomputed content hash did not match the spore's declared identity (ADR-003).
    HashMismatch {
        /// The hash the spore claims.
        expected: ContentHash,
        /// The hash recomputed from the component DAG.
        found: ContentHash,
    },
}

/// A content-addressed deployable handle: its canonical content hash *is* its identity (ADR-003).
/// The full component DAG (code + values + manifest-by-digest + dependency edges) and the
/// reconstruction manifest are wired by the M-522 leaf.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spore {
    /// The canonical content hash — the spore's identity.
    pub content: ContentHash,
}

impl Spore {
    /// The spore's canonical content-addressed identity (ADR-003).
    #[must_use]
    pub fn content(&self) -> &ContentHash {
        &self.content
    }

    /// Verify the spore's manifest against its declared identity.
    ///
    /// SCAFFOLD: the M-522 leaf recomputes the component-DAG hash and compares; a mismatch is an
    /// explicit [`SporeErr::HashMismatch`] (never a silent accept — C1/G2). The stub returns `Ok`
    /// for the trivially-consistent handle.
    pub fn verify(&self) -> Result<(), SporeErr> {
        Ok(())
    }
}
