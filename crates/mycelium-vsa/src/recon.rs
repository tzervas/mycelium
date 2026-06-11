//! **Reconstruction execution** over a [`ReconInfo`] manifest (M-260; RFC-0003 §6; FR-S4).
//!
//! The kernel carries the manifest *data type* (`mycelium_core::ReconInfo`); this module is the
//! submodule-side executor: [`reconstruct_role`] performs **true compositional reconstruction** —
//! unbind the record by a role named in the manifest's recipe, then clean up against the
//! codebook — recovering a *novel combination* never stored as an atom (the §6 exit criterion,
//! VSA's defining capability over a hash table). Everything is inspectable: the recipe names the
//! roles, the codebook is content-addressed, and the returned [`Match`] carries the cleanup
//! confidence/margin, thresholded against the manifest's own `cleanup_threshold` — a
//! below-threshold retrieval is an explicit error, never a silent low-quality answer (G2).

use mycelium_core::{DecodeProcedure, ReconInfo, ReconMode, Value};

use crate::{CleanupMemory, Match, VsaError, VsaModel};

/// Compositionally reconstruct the filler bound under `role` inside `record`, following the
/// manifest: requires a `CompositionalReconstruction` manifest with a `Cleanup` decode whose
/// recipe names `role`; unbinds `record` by the role atom and cleans the noisy result up against
/// `memory` (which must hold the manifest's codebook atoms). Explicit refusals: a non-matching
/// model/dim, a mode/procedure the manifest does not declare, an unknown role, and a retrieval
/// below the manifest's threshold.
pub fn reconstruct_role<M: VsaModel>(
    model: &M,
    manifest: &ReconInfo,
    record: &Value,
    role: &str,
    role_atom: &Value,
    memory: &CleanupMemory,
) -> Result<Match, VsaError> {
    if manifest.model() != model.model_id() {
        return Err(VsaError::NotThisModel {
            expected: model.model_id(),
        });
    }
    let recipe = match (manifest.mode(), manifest.recipe()) {
        (ReconMode::CompositionalReconstruction, Some(r)) => r,
        // An indexed-retrieval manifest cannot reconstruct compositionally — refusing is exactly
        // the §6 distinction made operational.
        _ => return Err(VsaError::NotCompositional),
    };
    if !recipe.roles.iter().any(|r| r == role) {
        return Err(VsaError::UnknownRole {
            role: role.to_owned(),
        });
    }
    let threshold = match (
        manifest.decode().procedure,
        manifest.decode().cleanup_threshold,
    ) {
        (DecodeProcedure::Cleanup, Some(t)) => t,
        // Resonator decoding is Phase-3 exploratory (FR-C2) — explicit, not a fallback.
        _ => return Err(VsaError::NotCompositional),
    };

    let record_hv = hv_payload(model, manifest.dim(), record)?;
    let role_hv = hv_payload(model, manifest.dim(), role_atom)?;
    let noisy = model.unbind(record_hv, role_hv)?;
    let hit = memory.cleanup(&noisy, model).ok_or(VsaError::EmptyBundle)?;
    if hit.confidence < threshold {
        return Err(VsaError::BelowCleanupThreshold {
            confidence: hit.confidence,
            threshold,
        });
    }
    Ok(hit)
}

fn hv_payload<'a, M: VsaModel>(model: &M, dim: u32, v: &'a Value) -> Result<&'a [f64], VsaError> {
    match (v.repr(), v.payload()) {
        (
            mycelium_core::Repr::Vsa {
                model: m, dim: d, ..
            },
            mycelium_core::Payload::Hypervector(h),
        ) if m == model.model_id() && *d == dim => Ok(h),
        _ => Err(VsaError::NotThisModel {
            expected: model.model_id(),
        }),
    }
}
