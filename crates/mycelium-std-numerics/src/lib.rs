//! `std.numerics` — the honest ε/δ carrier and meet-composition surface (M-512, issue #153).
//!
//! `std.numerics` is the library home for *carrying a value together with its `Meta`-attached
//! `{Bound, GuaranteeStrength}`* ([`Approx<T>`]) and for the **meet-composition /
//! refuse-without-a-rule** posture that keeps bounds honest. The verified ε/δ kernels live in
//! `mycelium-numerics` (ADR-010); this module is their **ergonomic, never-upgrading surface**
//! (KC-3 — it adds no trusted bound algebra, it consumes it).
//!
//! # Honesty crux (C2 / VR-5)
//!
//! A helper's guarantee tag is exactly what its basis supports and is **never upgraded**: strength
//! composes by [`GuaranteeStrength::meet`] (weakest-wins), bounds propagate with **outward
//! (directed) rounding**, and an op lacking a sound rule **refuses** (`Result`/`Option`) rather than
//! fabricating a bound. `Proven` is constructible **only** via a checked-theorem witness.
//!
//! Design spec: `docs/spec/stdlib/numerics.md`; ADR-010/011; task M-512, issue #153.
//!
//! ## Scaffold status (SCAFFOLD — M-512 leaf to complete)
//!
//! Stub surface only: the [`Approx<T>`] carrier + constructor/`combine`/`explain` signatures so the
//! workspace builds. The M-512 leaf agent fills in: outward-rounding bound propagation, the
//! `ProvenThm`-witnessed `proven` constructor, ε-constant citation from `mycelium-numerics` (restate
//! none — NFR-N2), the §4.5 guarantee matrix as checked data, and the meet-never-upgrades /
//! outward-rounding property tests.
#![forbid(unsafe_code)]

use mycelium_core::{Bound, GuaranteeStrength};

/// A thin view pairing a value with its `{Bound, strength}` (RFC-0001 §4.3 `Meta`) — **not** a new
/// numeric type and **no kernel change** (FR-N1 / KC-3).
#[derive(Debug, Clone, PartialEq)]
pub struct Approx<T> {
    /// The carried value.
    pub value: T,
    /// The error/probability bound certifying `value`.
    pub bound: Bound,
    /// The honest guarantee strength (never upgraded; VR-5).
    pub strength: GuaranteeStrength,
}

impl<T> Approx<T> {
    /// Construct a `Declared`-strength approximation (always-flagged, user-asserted bound).
    #[must_use]
    pub fn declared(value: T, bound: Bound) -> Self {
        Self {
            value,
            bound,
            strength: GuaranteeStrength::Declared,
        }
    }

    /// Construct an `Empirical`-strength approximation (empirically-fit bound).
    #[must_use]
    pub fn empirical(value: T, bound: Bound) -> Self {
        Self {
            value,
            bound,
            strength: GuaranteeStrength::Empirical,
        }
    }

    /// The honest strength of this approximation.
    #[must_use]
    pub fn strength(&self) -> GuaranteeStrength {
        self.strength
    }
}
