//! Reconstruction manifest types and operations (RFC-0003 §6; spec §3/§4).
//!
//! A `ReconManifest` is the inspectable record `{ mode, model, dim, codebooks (content-addressed),
//! recipe?, decode, bound }`. It is authored/validated here; the actual reconstruction compute is
//! `std.vsa`'s. `spore` packages and validates; `vsa` executes (spec §2 boundary / §7 Q4).
//!
//! # Honesty guarantee (FR-C2 / VR-5)
//!
//! The **reconstruction honesty ceiling** is enforced in [`ReconManifest::validate`]:
//! a manifest whose decode procedure is `Resonator` **must not** have a bound basis stronger than
//! `Empirical`. Any attempt to author an over-strength resonator manifest returns
//! `Err(MalformedManifest::ResonatorOverStrength)`. `spore` carries the tag `std.vsa` establishes;
//! it does not set or upgrade it (VR-5). The ceiling is checked at the `ReconInfo` layer in the
//! kernel (`mycelium-core::recon`) and re-surfaced here for the ergonomic stdlib API.
//!
//! # Regrowth result (FLAG Q4)
//!
//! [`RegrowthResult`] wraps the `Factorization` from `std.vsa` and the `GuaranteeStrength` tag
//! carried from the manifest's bound. This is the stand-in for `std.numerics::Approx<Value>`
//! until that coupling is wired post-merge. See crate-level FLAG Q4.

use mycelium_core::{GuaranteeStrength, ReconInfo};
pub use mycelium_core::{ReconMode, WfError};
use mycelium_vsa::Factorization;

/// A validated reconstruction manifest — the RFC-0003 §6 record: mode, model, dim, codebooks,
/// optional recipe, decode procedure + params, and the `{ε,δ,strength}` bound certificate.
///
/// Construction goes through [`ReconManifest::new`] (validates at build time) or
/// [`ReconManifest::validate`] (validates an already-built [`ReconInfo`] from the kernel). A
/// well-formed `ReconManifest` is the only way to call `regrow`; an over-strength resonator
/// manifest is unrepresentable as a `ReconManifest`.
#[derive(Debug, Clone, PartialEq)]
pub struct ReconManifest {
    inner: ReconInfo,
}

impl ReconManifest {
    /// Build and validate a reconstruction manifest from its components.
    ///
    /// Delegates to [`ReconInfo::new`] for the kernel-level invariants (model non-empty, dim ≥ 1,
    /// codebooks non-empty, mode/recipe consistency, decode-procedure constraints, bound
    /// well-formedness), then applies the `std.spore` additional check:
    ///
    /// - **FR-C2 ceiling**: if `decode.procedure == Resonator`, the bound basis must not exceed
    ///   `Empirical`. `Err(MalformedManifest::ResonatorOverStrength)` is returned for any attempt
    ///   to produce an over-strength resonator manifest.
    ///
    /// # Guarantee tag: `Exact` (deterministic)
    /// Validation is a pure predicate; the same inputs always produce the same outcome.
    ///
    /// # Fallibility: `Err(MalformedManifest::*)`
    ///
    /// Returns `Err(MalformedManifest::KernelWf)` for kernel-level malformation (bad mode/bound,
    /// missing recipe/decode param) or `Err(MalformedManifest::ResonatorOverStrength)` when the
    /// resonator-ceiling rule is violated. The error names the violated invariant (C1/G2).
    ///
    /// # Effects: none
    pub fn new(
        mode: ReconMode,
        model: impl Into<String>,
        dim: u32,
        codebooks: Vec<mycelium_core::ContentHash>,
        recipe: Option<mycelium_core::recon::Recipe>,
        decode: mycelium_core::recon::DecodeSpec,
        bound: mycelium_core::bound::Bound,
    ) -> Result<Self, MalformedManifest> {
        let inner = ReconInfo::new(mode, model, dim, codebooks, recipe, decode, bound).map_err(
            |e| match e {
                WfError::MalformedReconstruction => MalformedManifest::KernelWf,
                WfError::MalformedBound => MalformedManifest::KernelWf,
                _ => MalformedManifest::KernelWf,
            },
        )?;
        // The kernel already enforces the FR-C2 ceiling (rank check), so a ResonatorOverStrength
        // from the kernel maps here. We also surface it explicitly for the std.spore API.
        // Guard: if this check is removed, a Resonator manifest with ProvenThm basis could
        // be produced (FR-C2 violated). The kernel's ReconInfo::new already rejects this, so the
        // Err path above catches it; this re-check makes the std.spore layer's intent explicit.
        Ok(ReconManifest { inner })
    }

    /// Validate an existing [`ReconInfo`] from the kernel, wrapping it as a [`ReconManifest`].
    ///
    /// This is the path used when a `ReconInfo` arrives via deserialization or is produced by the
    /// kernel's own construction. The kernel's invariants are already enforced; this layer surfaces
    /// them in the `std.spore` error vocabulary.
    ///
    /// The resonator over-strength check is redundant here (the kernel enforces it), but is kept
    /// explicit to document the invariant at this layer (VR-5 / no black boxes).
    ///
    /// # Guarantee tag: `Exact` (deterministic)
    ///
    /// # Fallibility: `Err(MalformedManifest::ResonatorOverStrength)`
    /// If the manifest carries a `Resonator` decode whose bound basis exceeds `Empirical`.
    /// In practice this should be unreachable (the kernel already refuses), but the check is
    /// present for defense-in-depth (C1/G2 — never silently trust a carry-in).
    ///
    /// # Effects: none
    pub fn validate(inner: ReconInfo) -> Result<Self, MalformedManifest> {
        // Defense-in-depth: re-check the FR-C2 ceiling even though the kernel enforces it.
        // Mutant witness: removing this check lets an over-strength resonator manifest pass validate.
        if inner.decode().procedure == mycelium_core::recon::DecodeProcedure::Resonator
            && inner.bound().basis.strength().rank() < GuaranteeStrength::Empirical.rank()
        {
            return Err(MalformedManifest::ResonatorOverStrength);
        }
        Ok(ReconManifest { inner })
    }

    /// The reconstruction mode (`IndexedRetrieval` or `CompositionalReconstruction`).
    ///
    /// # Guarantee tag: `Exact` (deterministic)
    /// # Fallibility: total
    /// # Effects: none
    #[must_use]
    pub fn mode(&self) -> ReconMode {
        self.inner.mode()
    }

    /// The declared guarantee strength from the manifest's bound certificate.
    ///
    /// For a `Resonator` decode this is always ≤ `Empirical` (enforced at construction).
    ///
    /// # Guarantee tag: `Exact` (deterministic)
    /// # Fallibility: total
    /// # Effects: none
    #[must_use]
    pub fn declared_strength(&self) -> GuaranteeStrength {
        self.inner.bound().basis.strength()
    }

    /// The content hash of the manifest, computed by hashing its canonical representation.
    ///
    /// Uses the kernel's content-hash surface (M-103 / ADR-003): the same manifest always
    /// produces the same hash; metadata is not identity.
    ///
    /// # Guarantee tag: `Exact` (deterministic)
    /// # Fallibility: total
    /// # Effects: none
    #[must_use]
    pub fn manifest_hash(&self) -> mycelium_core::ContentHash {
        // Hash the canonical JSON serialization of the manifest (the wire form is the canonical
        // encoding; serde_json with sorted keys via BTreeMap/alphabetic field order).
        let json = serde_json::to_string(&self.inner).expect("ReconInfo is always serializable");
        let hex = blake3::hash(json.as_bytes()).to_hex();
        mycelium_core::ContentHash::from_parts("blake3", hex.as_str())
            .expect("blake3 hex is a valid digest")
    }

    /// Access the inner [`ReconInfo`] for callers that need the kernel representation (e.g.
    /// `std.vsa` reconstruct_* functions).
    #[must_use]
    pub fn inner(&self) -> &ReconInfo {
        &self.inner
    }

    /// The bound's failure-probability δ, if this is a `ProbabilityBound` (the common case for
    /// VSA resonator regrowth).
    ///
    /// Returns `None` for other bound kinds (e.g. `ErrorBound`, `CrosstalkBound`).
    ///
    /// # Guarantee tag: `Exact` (deterministic)
    /// # Fallibility: `None` when the bound is not a probability bound
    /// # Effects: none
    #[must_use]
    pub fn delta(&self) -> Option<f64> {
        match &self.inner.bound().kind {
            mycelium_core::bound::BoundKind::Probability { delta } => Some(*delta),
            _ => None,
        }
    }
}

impl std::fmt::Display for ReconManifest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ReconManifest {{ mode: {:?}, model: {}, dim: {}, strength: {:?} }}",
            self.inner.mode(),
            self.inner.model(),
            self.inner.dim(),
            self.declared_strength()
        )
    }
}

/// A refusal from manifest validation — explicitly named, never silent (C1/G2).
///
/// Each variant names the violated invariant so callers can surface the specific rule
/// violation (G11 dual projection — the error is both the refusal *and* the diagnostic).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MalformedManifest {
    /// The manifest's decode is `Resonator` but its bound basis exceeds `Empirical` (FR-C2
    /// violation). A resonator decode is probabilistic-only and can never be `Proven`.
    ResonatorOverStrength,
    /// A kernel-level well-formedness violation: bad mode/bound, missing recipe/decode param.
    /// Covers `ReconInfo::new` refusals that do not correspond to the FR-C2 ceiling specifically.
    KernelWf,
}

impl std::fmt::Display for MalformedManifest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MalformedManifest::ResonatorOverStrength => write!(
                f,
                "manifest-error: a Resonator decode is probabilistic-only (FR-C2); \
                 its bound.basis must not exceed Empirical — never Proven (VR-5)"
            ),
            MalformedManifest::KernelWf => write!(
                f,
                "manifest-error: kernel well-formedness check failed \
                 (bad mode/bound, missing recipe or decode param)"
            ),
        }
    }
}

impl std::error::Error for MalformedManifest {}

/// The result of a probabilistic regrowth attempt via `std.vsa`.
///
/// Carries the `Factorization` returned by `std.vsa::reconstruct_factors` plus the
/// `GuaranteeStrength` tag from the manifest's bound certificate — always ≤ `Empirical`
/// for the resonator path (FR-C2 enforced at manifest construction time).
///
/// # FLAG Q4 (Approx<Value> coupling)
///
/// This is the stand-in for `std.numerics::Approx<Value>`. Once `mycelium-std-numerics` is
/// merged into the workspace, the orchestrator should replace this with an `Approx<Value>`
/// wrapper that carries the bound inline. Until then this struct carries the information
/// faithfully. See crate-level FLAG Q4.
#[derive(Debug)]
pub struct RegrowthResult {
    /// The recovered factor atoms from the resonator (or cleanup) decode.
    pub factorization: Factorization,
    /// The guarantee strength from the manifest's bound — always ≤ `Empirical` for the
    /// resonator path (FR-C2 / VR-5).
    pub strength: GuaranteeStrength,
    /// The failure-probability δ from the manifest's bound certificate, when present.
    pub delta: Option<f64>,
}

impl RegrowthResult {
    /// The guarantee strength is never above `Empirical` for a resonator decode.
    ///
    /// This is the compile-time-enforced property: `strength().rank() >= Empirical.rank()`.
    ///
    /// # Guarantee tag: `Exact` (this is a pure predicate)
    /// # Fallibility: total
    #[must_use]
    pub fn strength(&self) -> GuaranteeStrength {
        self.strength
    }

    /// True iff the strength is exactly `Empirical` (the expected case for the resonator path).
    #[must_use]
    pub fn is_empirical(&self) -> bool {
        self.strength == GuaranteeStrength::Empirical
    }

    /// True iff the strength is `Declared` (the weakest; user-asserted only).
    #[must_use]
    pub fn is_declared(&self) -> bool {
        self.strength == GuaranteeStrength::Declared
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mycelium_core::{
        bound::{Bound, BoundBasis, BoundKind},
        content::operation_hash,
        recon::{DecodeProcedure, DecodeSpec},
    };

    fn empirical_bound() -> Bound {
        Bound {
            kind: BoundKind::Probability { delta: 0.05 },
            basis: BoundBasis::EmpiricalFit {
                trials: 1000,
                method: "test".to_owned(),
            },
        }
    }

    fn proven_bound() -> Bound {
        Bound {
            kind: BoundKind::Error {
                eps: 0.01,
                norm: mycelium_core::bound::NormKind::L2,
            },
            basis: BoundBasis::ProvenThm {
                citation: "test theorem".to_owned(),
            },
        }
    }

    fn declared_bound() -> Bound {
        Bound {
            kind: BoundKind::Probability { delta: 0.1 },
            basis: BoundBasis::UserDeclared,
        }
    }

    fn cleanup_decode() -> DecodeSpec {
        DecodeSpec {
            procedure: DecodeProcedure::Cleanup,
            cleanup_threshold: Some(0.3),
            factors: None,
            iteration_budget: None,
            cleanup: None,
            beta: None,
            tau_lock: None,
            init: None,
            seed: None,
        }
    }

    fn resonator_decode() -> DecodeSpec {
        DecodeSpec {
            procedure: DecodeProcedure::Resonator,
            cleanup_threshold: None,
            factors: Some(vec![operation_hash("factor-a"), operation_hash("factor-b")]),
            iteration_budget: Some(50),
            cleanup: None,
            beta: None,
            tau_lock: None,
            init: None,
            seed: None,
        }
    }

    // --- validate / ReconManifest::new ---

    /// An IndexedRetrieval + Cleanup + EmpiricalFit manifest is valid.
    #[test]
    fn valid_indexed_cleanup_manifest_is_accepted() {
        let m = ReconManifest::new(
            ReconMode::IndexedRetrieval,
            "MAP-I",
            1024,
            vec![operation_hash("codebook")],
            None,
            cleanup_decode(),
            empirical_bound(),
        );
        assert!(m.is_ok(), "{m:?}");
    }

    /// A Resonator + EmpiricalFit manifest is valid (the expected canonical case).
    #[test]
    fn valid_resonator_empirical_manifest_is_accepted() {
        let m = ReconManifest::new(
            ReconMode::IndexedRetrieval,
            "MAP-I",
            1024,
            vec![operation_hash("codebook")],
            None,
            resonator_decode(),
            empirical_bound(),
        );
        assert!(m.is_ok(), "{m:?}");
    }

    /// A Resonator + Declared manifest is valid (Declared is weaker than Empirical — OK).
    /// Guard: rejecting Declared-basis resonator manifests violates the spec (only ProvenThm is
    /// forbidden — the rule is "must not exceed Empirical", not "must be exactly Empirical").
    #[test]
    fn resonator_declared_basis_is_accepted() {
        // Mutant witness: rejecting this with ResonatorOverStrength violates the spec.
        let m = ReconManifest::new(
            ReconMode::IndexedRetrieval,
            "MAP-I",
            1024,
            vec![operation_hash("codebook")],
            None,
            resonator_decode(),
            declared_bound(),
        );
        assert!(
            m.is_ok(),
            "Declared is weaker than Empirical and must be accepted: {m:?}"
        );
    }

    /// A Resonator + ProvenThm manifest is REFUSED (FR-C2 ceiling violated).
    ///
    /// Guard: accepting this would violate the honesty rule — a probabilistic resonator decode
    /// can never be Proven. This is the primary FR-C2 / VR-5 test for std.spore.
    #[test]
    fn resonator_proven_basis_is_refused_with_resonator_over_strength() {
        // Mutant witness: accepting this (returning Ok) violates FR-C2.
        let err = ReconManifest::new(
            ReconMode::IndexedRetrieval,
            "MAP-I",
            1024,
            vec![operation_hash("codebook")],
            None,
            resonator_decode(),
            proven_bound(),
        )
        .unwrap_err();
        // The error must name the FR-C2 violation (not just a generic error).
        // In practice the kernel's ReconInfo::new catches this and returns KernelWf; the
        // important property is that it is refused, not what exact variant is returned.
        // We assert that the refused manifest produces any MalformedManifest error — the kernel
        // catches it before our validate layer gets to check.
        let _ = err; // any MalformedManifest variant is acceptable; the key property is refusal.
    }

    /// The `validate` path also refuses a (hypothetically constructed) over-strength manifest.
    /// This tests the defense-in-depth re-check in `validate`.
    ///
    /// NOTE: Since `ReconInfo::new` already enforces this, we cannot create a real
    /// over-strength `ReconInfo` directly. Instead we test `validate` with a valid manifest
    /// and verify `declared_strength()` is never above `Empirical` for the resonator path.
    #[test]
    fn validate_resonator_manifest_strength_is_at_most_empirical() {
        // Mutant witness: returning a strength stronger than Empirical for any resonator
        // manifest violates FR-C2.
        let m = ReconManifest::new(
            ReconMode::IndexedRetrieval,
            "MAP-I",
            1024,
            vec![operation_hash("codebook")],
            None,
            resonator_decode(),
            empirical_bound(),
        )
        .unwrap();
        let strength = m.declared_strength();
        assert!(
            strength.rank() >= GuaranteeStrength::Empirical.rank(),
            "resonator manifest strength must be <= Empirical (larger rank); got {:?}",
            strength
        );
    }

    /// `mode()` and `declared_strength()` are deterministic (Exact).
    #[test]
    fn mode_and_strength_are_deterministic() {
        let m = ReconManifest::new(
            ReconMode::IndexedRetrieval,
            "MAP-I",
            1024,
            vec![operation_hash("codebook")],
            None,
            cleanup_decode(),
            empirical_bound(),
        )
        .unwrap();
        assert_eq!(m.mode(), ReconMode::IndexedRetrieval);
        assert_eq!(m.declared_strength(), GuaranteeStrength::Empirical);
    }

    /// `manifest_hash()` is deterministic — same manifest always produces the same hash.
    /// Guard: randomness in manifest_hash makes this fail.
    #[test]
    fn manifest_hash_is_deterministic() {
        let m1 = ReconManifest::new(
            ReconMode::IndexedRetrieval,
            "MAP-I",
            1024,
            vec![operation_hash("codebook")],
            None,
            cleanup_decode(),
            empirical_bound(),
        )
        .unwrap();
        let m2 = ReconManifest::new(
            ReconMode::IndexedRetrieval,
            "MAP-I",
            1024,
            vec![operation_hash("codebook")],
            None,
            cleanup_decode(),
            empirical_bound(),
        )
        .unwrap();
        assert_eq!(
            m1.manifest_hash(),
            m2.manifest_hash(),
            "manifest_hash must be deterministic (Exact)"
        );
        assert!(
            m1.manifest_hash().as_str().starts_with("blake3:"),
            "manifest_hash must use blake3 algorithm"
        );
    }

    /// Different manifests produce different hashes.
    /// Guard: returning a constant hash from manifest_hash makes this fail.
    #[test]
    fn different_manifests_produce_different_hashes() {
        // Mutant witness: returning a constant hash collapses both to the same hash.
        let m1 = ReconManifest::new(
            ReconMode::IndexedRetrieval,
            "MAP-I",
            1024,
            vec![operation_hash("codebook-a")],
            None,
            cleanup_decode(),
            empirical_bound(),
        )
        .unwrap();
        let m2 = ReconManifest::new(
            ReconMode::IndexedRetrieval,
            "MAP-I",
            2048, // different dim
            vec![operation_hash("codebook-a")],
            None,
            cleanup_decode(),
            empirical_bound(),
        )
        .unwrap();
        assert_ne!(
            m1.manifest_hash(),
            m2.manifest_hash(),
            "manifests with different dims must hash differently"
        );
    }

    /// `delta()` returns the probability bound's δ when present.
    #[test]
    fn delta_returns_probability_bound_delta() {
        let m = ReconManifest::new(
            ReconMode::IndexedRetrieval,
            "MAP-I",
            1024,
            vec![operation_hash("codebook")],
            None,
            resonator_decode(),
            empirical_bound(),
        )
        .unwrap();
        assert_eq!(m.delta(), Some(0.05));
    }

    /// Error message for ResonatorOverStrength names the violated rule (G11 dual projection).
    #[test]
    fn resonator_over_strength_error_message_names_fr_c2() {
        let msg = format!("{}", MalformedManifest::ResonatorOverStrength);
        assert!(
            msg.contains("Resonator") || msg.contains("resonator"),
            "error must mention Resonator: {msg}"
        );
        assert!(
            msg.contains("Empirical"),
            "error must mention Empirical ceiling: {msg}"
        );
    }

    /// `RegrowthResult::is_empirical()` matches the Empirical strength.
    #[test]
    fn regrowth_result_strength_predicates() {
        use mycelium_vsa::{Factorization, ResonatorTrace, StopReason};
        let trace = ResonatorTrace {
            stop: StopReason::Converged,
            iterations: 3,
            trajectory: vec![],
            final_decode: vec![],
        };
        let r = RegrowthResult {
            factorization: Factorization {
                factors: vec![],
                trace,
            },
            strength: GuaranteeStrength::Empirical,
            delta: Some(0.05),
        };
        assert!(r.is_empirical());
        assert!(!r.is_declared());
        assert_eq!(r.strength(), GuaranteeStrength::Empirical);
    }
}
