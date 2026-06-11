//! **FHRR** — Fourier Holographic Reduced Representations (frequency-domain / phasor) (M-241;
//! RFC-0003 §4; T1.2).
//!
//! Hypervector components are **phase angles** `θᵢ ∈ (−π, π]` standing for the unit phasors
//! `e^{iθᵢ}` (the natural `Vec<f64>` encoding of unit-magnitude complex components). The algebra:
//! - **bind** = elementwise **phase addition** (complex multiplication of phasors) — algebraic
//!   and deterministic; **unbind** = phase subtraction (conjugate multiplication), the
//!   approximate-inverse role in use (decoding from superpositions needs cleanup) — tagged
//!   **`Empirical`**, the same weak-link assignment as HRR (RFC-0003 §4 / T1.2; never upgraded
//!   even though pure-pair recovery is near-exact, the matrix is normative).
//! - **bundle** = per-component **complex sum renormalized to a phasor** (`arg Σ e^{iθ}`) —
//!   lossy by construction (magnitude is discarded); **`Empirical`** (RFC-0003 §4). A component
//!   whose phasor sum has (near-)zero magnitude has no defined phase — an explicit
//!   [`VsaError::DegenerateBundleComponent`], never an arbitrary pick.
//! - **permute** = cyclic shift — **`Exact`**.
//! - **similarity** = mean `cos(θa − θb)` (the real part of the normalized Hermitian inner
//!   product) in `[-1, 1]`.

use mycelium_core::{operation_hash, GuaranteeStrength, Provenance, Value};

use crate::wrap::{hv_of, rotate, wrap, wrap_exact};
use crate::{EmpiricalProfile, VsaError, VsaModel, VsaOp};

/// The trial-validated regime backing the Value-level FHRR unbind's `Empirical` δ
/// (`tests/empirical_profiles.rs` runs exactly these trials).
pub const FHRR_UNBIND_PROFILE: EmpiricalProfile = EmpiricalProfile {
    max_items: 1,
    odd_items_only: false,
    min_dim: 256,
    delta: 1e-2,
    trials: 10_000,
    method: "Monte-Carlo bind→unbind→cleanup recovery (uniform phasor atoms, single bind factor, \
             codebook ≤ 16, d ≥ 256)",
};

/// Wrap an angle to `(−π, π]`.
fn wrap_phase(theta: f64) -> f64 {
    let t = theta.rem_euclid(std::f64::consts::TAU);
    if t > std::f64::consts::PI {
        t - std::f64::consts::TAU
    } else {
        t
    }
}

/// The FHRR model at a fixed dimensionality.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Fhrr {
    /// Hypervector dimensionality (number of phasor components).
    pub dim: u32,
}

impl Fhrr {
    /// An FHRR model of dimension `dim`.
    #[must_use]
    pub fn new(dim: u32) -> Self {
        Fhrr { dim }
    }

    fn check_len(&self, v: &[f64]) -> Result<(), VsaError> {
        if v.len() == self.dim as usize {
            Ok(())
        } else {
            Err(VsaError::DimMismatch {
                expected: self.dim as usize,
                got: v.len(),
            })
        }
    }

    /// Components must be phases in `(−π, π]` — anything else is outside the phasor alphabet
    /// (refused, never silently wrapped: an out-of-range payload contradicts its encoding).
    fn check_phases(v: &[f64]) -> Result<(), VsaError> {
        match v
            .iter()
            .position(|&t| !t.is_finite() || t <= -std::f64::consts::PI || t > std::f64::consts::PI)
        {
            Some(index) => Err(VsaError::NonAlphabetComponent { index }),
            None => Ok(()),
        }
    }

    /// Value-level `bind` (deterministic phasor algebra).
    pub fn bind_values(&self, a: &Value, b: &Value) -> Result<Value, VsaError> {
        let out = self.bind(
            hv_of(self.model_id(), self.dim, a)?,
            hv_of(self.model_id(), self.dim, b)?,
        )?;
        wrap_exact(
            self.model_id(),
            self.dim,
            out,
            "vsa.fhrr.bind",
            vec![a.content_hash(), b.content_hash()],
        )
    }

    /// Value-level **`Empirical` unbind** (the RFC-0003 §4 weak-link tag, like HRR): the decoded
    /// vector carries the [`FHRR_UNBIND_PROFILE`] δ and is completed through a
    /// [`CleanupMemory`](crate::CleanupMemory). Gated to the validated regime: `a` must be a
    /// single `vsa.fhrr.bind` product.
    pub fn unbind_values(&self, a: &Value, b: &Value) -> Result<Value, VsaError> {
        FHRR_UNBIND_PROFILE.check(1, self.dim)?;
        match a.meta().provenance() {
            Provenance::Derived { op, .. } if op == &operation_hash("vsa.fhrr.bind") => {}
            _ => {
                return Err(VsaError::OutsideEmpiricalProfile {
                    detail: "input is not a single vsa.fhrr.bind product (the validated \
                             single-factor regime)"
                        .to_owned(),
                })
            }
        }
        let out = self.unbind(
            hv_of(self.model_id(), self.dim, a)?,
            hv_of(self.model_id(), self.dim, b)?,
        )?;
        wrap(
            self.model_id(),
            self.dim,
            out,
            "vsa.fhrr.unbind",
            vec![a.content_hash(), b.content_hash()],
            GuaranteeStrength::Empirical,
            Some(FHRR_UNBIND_PROFILE.bound()),
        )
    }
}

impl VsaModel for Fhrr {
    fn model_id(&self) -> &'static str {
        "FHRR"
    }

    fn self_inverse(&self) -> bool {
        false
    }

    fn intrinsic_guarantee(&self, op: VsaOp) -> GuaranteeStrength {
        match op {
            // Algebraic, deterministic phasor ops.
            VsaOp::Bind | VsaOp::Permute => GuaranteeStrength::Exact,
            // The weak-link assignment (RFC-0003 §4 / T1.2) — normative, never upgraded.
            VsaOp::Unbind | VsaOp::Bundle => GuaranteeStrength::Empirical,
        }
    }

    fn bind(&self, a: &[f64], b: &[f64]) -> Result<Vec<f64>, VsaError> {
        self.check_len(a)?;
        self.check_len(b)?;
        Self::check_phases(a)?;
        Self::check_phases(b)?;
        Ok(a.iter().zip(b).map(|(x, y)| wrap_phase(x + y)).collect())
    }

    fn unbind(&self, a: &[f64], b: &[f64]) -> Result<Vec<f64>, VsaError> {
        self.check_len(a)?;
        self.check_len(b)?;
        Self::check_phases(a)?;
        Self::check_phases(b)?;
        Ok(a.iter().zip(b).map(|(x, y)| wrap_phase(x - y)).collect())
    }

    fn bundle(&self, items: &[&[f64]]) -> Result<Vec<f64>, VsaError> {
        if items.is_empty() {
            return Err(VsaError::EmptyBundle);
        }
        for item in items {
            self.check_len(item)?;
            Self::check_phases(item)?;
        }
        let mut out = Vec::with_capacity(self.dim as usize);
        for index in 0..self.dim as usize {
            let re: f64 = items.iter().map(|v| v[index].cos()).sum();
            let im: f64 = items.iter().map(|v| v[index].sin()).sum();
            // A vanished phasor sum has no phase: explicit, never an arbitrary pick (G2).
            if (re * re + im * im).sqrt() < 1e-9 {
                return Err(VsaError::DegenerateBundleComponent { index });
            }
            out.push(wrap_phase(im.atan2(re)));
        }
        Ok(out)
    }

    fn permute(&self, a: &[f64], shift: i64) -> Result<Vec<f64>, VsaError> {
        self.check_len(a)?;
        Ok(rotate(a, shift))
    }

    fn unpermute(&self, a: &[f64], shift: i64) -> Result<Vec<f64>, VsaError> {
        self.permute(a, -shift)
    }

    /// Mean `cos(θa − θb)` — the real part of the normalized Hermitian inner product of the
    /// phasor vectors, in `[-1, 1]`.
    fn similarity(&self, a: &[f64], b: &[f64]) -> f64 {
        if a.len() != b.len() || a.is_empty() {
            return 0.0;
        }
        a.iter().zip(b).map(|(x, y)| (x - y).cos()).sum::<f64>() / a.len() as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CleanupMemory;
    use mycelium_core::{Meta, Payload, Repr, SparsityClass};

    /// Deterministic uniform-phase atom (tiny LCG — house style).
    fn phasor_atom(dim: u32, seed: u64) -> Vec<f64> {
        let mut s = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(1);
        (0..dim)
            .map(|_| {
                s = s
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                let u = (s >> 11) as f64 / (1u64 << 53) as f64; // [0, 1)
                wrap_phase(std::f64::consts::TAU * u)
            })
            .collect()
    }

    fn hv_value(dim: u32, seed: u64) -> Value {
        Value::new(
            Repr::Vsa {
                model: "FHRR".to_owned(),
                dim,
                sparsity: SparsityClass::Dense,
            },
            Payload::Hypervector(phasor_atom(dim, seed)),
            Meta::exact(mycelium_core::Provenance::Root),
        )
        .unwrap()
    }

    const D: u32 = 256;

    #[test]
    fn bind_unbind_recovers_up_to_rounding() {
        let m = Fhrr::new(D);
        assert!(!m.self_inverse());
        let a = phasor_atom(D, 1);
        let b = phasor_atom(D, 2);
        let bound = m.bind(&a, &b).unwrap();
        let recovered = m.unbind(&bound, &b).unwrap();
        let sim = m.similarity(&recovered, &a);
        assert!(
            sim > 0.999,
            "pure-pair recovery should be near-exact: {sim}"
        );
        // Still tagged Empirical — the matrix is normative (never upgraded past it).
        assert_eq!(
            m.intrinsic_guarantee(VsaOp::Unbind),
            GuaranteeStrength::Empirical
        );
    }

    #[test]
    fn bundle_is_phasor_valued_and_member_similar() {
        let m = Fhrr::new(D);
        let items: Vec<Vec<f64>> = (0..3).map(|i| phasor_atom(D, 30 + i)).collect();
        let refs: Vec<&[f64]> = items.iter().map(Vec::as_slice).collect();
        let bundle = m.bundle(&refs).unwrap();
        assert!(bundle
            .iter()
            .all(|&t| t > -std::f64::consts::PI && t <= std::f64::consts::PI));
        let member = m.similarity(&bundle, &items[0]);
        let stranger = m.similarity(&bundle, &phasor_atom(D, 555));
        assert!(
            member > stranger + 0.2,
            "member {member} vs stranger {stranger}"
        );
    }

    #[test]
    fn degenerate_bundle_component_is_explicit() {
        let m = Fhrr::new(2);
        // Opposite phasors cancel exactly at every component.
        let a = vec![0.5, -1.0];
        let b = vec![
            wrap_phase(0.5 + std::f64::consts::PI),
            wrap_phase(-1.0 + std::f64::consts::PI),
        ];
        assert_eq!(
            m.bundle(&[&a, &b]),
            Err(VsaError::DegenerateBundleComponent { index: 0 })
        );
    }

    #[test]
    fn out_of_range_phases_are_refused() {
        let m = Fhrr::new(2);
        assert_eq!(
            m.bind(&[0.1, 7.0], &[0.2, 0.3]),
            Err(VsaError::NonAlphabetComponent { index: 1 })
        );
    }

    #[test]
    fn value_unbind_is_empirical_and_regime_gated() {
        let m = Fhrr::new(D);
        let a = hv_value(D, 1);
        let b = hv_value(D, 2);
        let bound = m.bind_values(&a, &b).unwrap();
        let noisy = m.unbind_values(&bound, &b).unwrap();
        assert_eq!(noisy.meta().guarantee(), GuaranteeStrength::Empirical);
        // Root provenance → outside the validated single-factor regime.
        assert!(matches!(
            m.unbind_values(&a, &b),
            Err(VsaError::OutsideEmpiricalProfile { .. })
        ));
    }

    #[test]
    fn unbind_then_cleanup_recovers_the_filler() {
        let m = Fhrr::new(D);
        let role = phasor_atom(D, 10);
        let filler = phasor_atom(D, 20);
        let bound = m.bind(&role, &filler).unwrap();
        let mut mem = CleanupMemory::new(D);
        mem.insert("filler", filler).unwrap();
        mem.insert("other", phasor_atom(D, 21)).unwrap();
        let noisy = m.unbind(&bound, &role).unwrap();
        let hit = mem.cleanup(&noisy, &m).unwrap();
        assert_eq!(hit.label, "filler");
        assert!(hit.confidence > 0.9);
    }
}
