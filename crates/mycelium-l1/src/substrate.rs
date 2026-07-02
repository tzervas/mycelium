//! The `Substrate` v0 value form (M-902; DN-71 **Model S** §4.1, maintainer-accepted 2026-07-02).
//!
//! A `Substrate{tag}` value is an **interpreter-level opaque affine handle** — a value-world citizen
//! of the L1 evaluator (the [`crate::eval::L1Value::Substrate`] variant), **not** a kernel value:
//!
//! - **not a kernel node and not a [`mycelium_core::Repr`]** — it names an *external resource*
//!   (RFC-0006 LR-8), not a value representation, so it grows no L0 `Node` and no `Repr` arm
//!   (KC-3; DN-71 §4.1; the elaborator already states the ground truth: `Substrate` "is not a
//!   representation type").
//! - **not content-addressed data** (ADR-003 identity is for values; a handle's identity is *not*
//!   its content — two acquisitions of the same external resource are two distinct handles).
//! - **creatable only through the acquiring surface** — [`SubstrateHandle::acquire`] records the
//!   acquisition provenance; there is **no literal** and no free/`Default` constructor in safe
//!   surface code.
//! - **inspectable, never a black box** (house rule 2): the `tag`, the opaque identity, and the
//!   acquisition provenance are [`SubstrateHandle::explain`]-visible; the resource *contents* stay
//!   opaque (they are the host's, not the value world's).
//!
//! # Scope of M-902 (creation · passage · inspection)
//! This module makes `Substrate` values **exist**, be **passed** (they flow through the evaluator's
//! ordinary value-binding machinery — `let`, argument passing, whole-value pattern binders), and be
//! **inspected**. The invalid states are unrepresentable or explicit errors — never a silent default
//! or a panic (G2/VR-5).
//!
//! # What is deliberately *not* here — the M-903 / M-904 seams
//! The **affine use-once enforcement** (the consumed-state transition + its never-silent runtime
//! backstop, DN-71 §4.2) is **M-903**; the **`consume` lowering** (DN-71 §4.3) is **M-904**. Neither
//! is built here. The consume/move seam — [`SubstrateHandle::try_consume`] — is left as an
//! **explicit, refusing** stub that names those staging owners; it never silently moves, no-ops, or
//! fabricates a transition (G2/VR-5). The surface `consume <expr>` correspondingly stays an explicit
//! refusal in the evaluator (see `crate::eval`), now naming the M-903/M-904 seam.

use std::sync::atomic::{AtomicU64, Ordering};

/// Process-unique source of opaque handle identities. A handle's identity is the *external resource*
/// it names, **not** its content (ADR-003 does not apply), so every [`SubstrateHandle::acquire`]
/// mints a fresh id — two acquisitions of the "same" resource are two distinct handles. Starts at 1
/// so `0` is never a valid handle id (a cheap never-silent sentinel if one is ever needed).
static NEXT_HANDLE_ID: AtomicU64 = AtomicU64::new(1);

/// How a [`SubstrateHandle`] was acquired — acquisition-provenance metadata (DN-71 §4.1; FLAG-9's
/// recommended v0 posture: carry the provenance, since it is what makes a handle *inspectable*
/// rather than aspirational). EXPLAIN-visible (house rule 2 — no black boxes).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubstrateProvenance {
    /// The acquiring operation — e.g. the `wild:<op>` host-call key of the quarantined `@std-sys`
    /// FFI floor today (RFC-0031 D1), or `graft(cap)` when R2 activates (`runtime.md` §API).
    pub acquired_via: String,
    /// A human-readable acquisition site/context for the EXPLAIN trail (e.g. the function name or a
    /// source description). Free-form; carried verbatim, never interpreted.
    pub site: String,
}

impl SubstrateProvenance {
    /// A provenance record naming the acquiring op and the acquisition site.
    #[must_use]
    pub fn new(acquired_via: impl Into<String>, site: impl Into<String>) -> Self {
        SubstrateProvenance {
            acquired_via: acquired_via.into(),
            site: site.into(),
        }
    }
}

/// An opaque, runtime-only **affine `Substrate` handle** (DN-71 Model S §4.1; M-902). It carries its
/// affine-resource `tag` (LR-8), an opaque host-handle **identity** (distinct per acquisition), and
/// its acquisition [`SubstrateProvenance`]. It is the payload of the
/// [`crate::eval::L1Value::Substrate`] value form.
///
/// **On `Clone` and affinity.** This type is `Clone` so the handle can ride the evaluator's ordinary
/// value-passing machinery (binding a `let`, passing an argument, a whole-value pattern binder all
/// clone the bound `L1Value`). Cloning preserves the **same identity** (`id`) — a clone is the *same*
/// resource, i.e. surface *passage*, not a second resource. Affinity (use-once) is **not** enforced
/// by making this Rust type non-`Clone`; it is a **checker** property — the static affine pass
/// (M-903) ensures no surface program moves a `Substrate` binding more than once along any path. The
/// Rust-level `Clone` is the passage mechanism; the affine discipline lives one layer up (DN-71 §4.2;
/// DN-33 §8.1 Q4 — the known-affine binding is owned-unique in the checker, not in the value type).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubstrateHandle {
    tag: String,
    id: u64,
    provenance: SubstrateProvenance,
}

impl SubstrateHandle {
    /// **Acquire** a fresh `Substrate` handle for `tag`, recording how it was acquired. This is the
    /// *only* way a handle comes to exist (DN-71 §4.1: no literal, no `Default`, no safe surface
    /// constructor). Each call mints a **fresh opaque identity**, so two acquisitions of the same
    /// external resource are two distinct handles (identity is the resource, not its content).
    #[must_use]
    pub fn acquire(tag: impl Into<String>, provenance: SubstrateProvenance) -> Self {
        SubstrateHandle {
            tag: tag.into(),
            id: NEXT_HANDLE_ID.fetch_add(1, Ordering::Relaxed),
            provenance,
        }
    }

    /// The affine-resource `tag` (the `Substrate{tag}` name — RFC-0006 LR-8).
    #[must_use]
    pub fn tag(&self) -> &str {
        &self.tag
    }

    /// The opaque host-handle **identity** — distinct per [`acquire`](Self::acquire), and **not**
    /// content-derived. Two handles are the same resource iff their ids are equal.
    #[must_use]
    pub fn id(&self) -> u64 {
        self.id
    }

    /// The acquisition [`SubstrateProvenance`] (EXPLAIN-visible; DN-71 §4.1 / FLAG-9).
    #[must_use]
    pub fn provenance(&self) -> &SubstrateProvenance {
        &self.provenance
    }

    /// A never-silent, one-line EXPLAIN description of this handle (house rule 2 — no black boxes):
    /// the `tag`, the opaque identity, and the acquisition provenance are shown. The resource
    /// *contents* stay opaque (they are the host's, not the value world's), so this describes the
    /// handle without pretending to expose what it names.
    #[must_use]
    pub fn explain(&self) -> String {
        format!(
            "Substrate{{{}}} #{} acquired via `{}` at {}",
            self.tag, self.id, self.provenance.acquired_via, self.provenance.site
        )
    }

    /// The **consume/move seam** for the affine construct (DN-71 §4.2 enforcement / §4.3 lowering).
    ///
    /// M-902 makes the handle *exist*, *pass*, and be *inspected*; the use-once **transition**
    /// (Live → Consumed) with its static affine check is **M-903**, and the `consume` **lowering** is
    /// **M-904**. Neither is built here, so this is an **explicit, never-silent refusal** naming the
    /// staging owners — never a silent no-op, a fabricated move, or a panic (G2/VR-5). M-903 replaces
    /// this stub with the checked affine transition.
    pub fn try_consume(&self) -> Result<Self, SubstrateError> {
        Err(SubstrateError::AffineTrackingUnstaged {
            tag: self.tag.clone(),
        })
    }
}

/// Why a `Substrate` operation was refused — always explicit (never-silent; G2/VR-5). The variant
/// set is closed: v0 supports only the create/pass/inspect surface, so the sole refusal is the
/// staged affine-move seam.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubstrateError {
    /// The affine use-once **move** (Live → Consumed) is not yet built — its static enforcement is
    /// **M-903** and the `consume` lowering is **M-904** (DN-71 Model S §4.2/§4.3). A v0 handle
    /// exists, is passed, and is inspected, but it cannot yet be *consumed*: this is an explicit
    /// refusal, never a silent move (G2). Names the tag of the handle whose consume was refused.
    AffineTrackingUnstaged {
        /// The `tag` of the handle whose consume/move was refused.
        tag: String,
    },
}

impl core::fmt::Display for SubstrateError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            SubstrateError::AffineTrackingUnstaged { tag } => write!(
                f,
                "consume of `Substrate{{{tag}}}` is staged: the M-902 value form exists, but the \
                 affine use-once move (static enforcement M-903; `consume` lowering M-904) is not \
                 built — an explicit refusal, never a silent move (DN-71 Model S §4.2/§4.3; VR-5)"
            ),
        }
    }
}

impl std::error::Error for SubstrateError {}
