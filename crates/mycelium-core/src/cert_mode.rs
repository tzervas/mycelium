//! The certification mode (RFC-0034 §3.1, §5) — the tunable certification *policy* a value was
//! produced under.
//!
//! Carried on every [`Meta`](crate::meta::Meta) as a **never-silent** tag (RFC-0034 §3.1) and
//! deliberately **excluded from the content hash**: it rides `Meta`, which RFC-0001 §4.6 excludes
//! wholesale, so switching modes never perturbs a value's content identity (ADR-003). That exclusion
//! is therefore by construction, not a special case — see the `content_hash` exclusion test.
//!
//! Two **first-class** modes — [`Fast`](CertMode::Fast) (the default) and
//! [`Certified`](CertMode::Certified) — with [`Balanced`](CertMode::Balanced) an optional
//! intermediate (RFC-0034 §5). The mode is *disclosure of how much certification ran*, ordered by
//! [`depth`](CertMode::depth) `Fast < Balanced < Certified`. It is **not** a guarantee strength and
//! never upgrades one (VR-5): a `Fast` value sits at the structural `Exact`/`Declared` tags and never
//! claims an `Empirical`/`Proven` it did not earn. The mode-gating of tag *computation* lands in
//! M-787; this leaf (M-786) introduces the type and the never-silent tag.

/// The active certification mode a value was produced under (RFC-0034). Default
/// [`Fast`](CertMode::Fast) — the project default (RFC-0034 §5).
///
/// The `serde` form is the bare string `"Fast" | "Balanced" | "Certified"` (mirroring
/// [`GuaranteeStrength`](crate::guarantee::GuaranteeStrength)).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum CertMode {
    /// **Fast** (default): no runtime certification machinery — cert-free, memory-safe, inspectable,
    /// and still deployable (spores survive a cert-off runtime, RFC-0034 §8). Provenance tags stay
    /// structural (`Exact`/`Declared`); `Empirical`/`Proven` are not computed (RFC-0034 §7; M-787).
    #[default]
    Fast,
    /// **Balanced** (intermediate): provenance tags propagated and swap certificates *emitted*, but
    /// **unchecked** (RFC-0034 §5).
    Balanced,
    /// **Certified**: the full, checked, certificate-backed auditable framework — today's all-on
    /// behaviour, engaged on request (RFC-0034 §5).
    Certified,
}

impl CertMode {
    /// All three modes, weakest-to-strongest certification depth — for exhaustive iteration in tests
    /// and tooling.
    pub const ALL: [CertMode; 3] = [CertMode::Fast, CertMode::Balanced, CertMode::Certified];

    /// Certification **depth**, `0` = [`Fast`](CertMode::Fast) (least) … `2` =
    /// [`Certified`](CertMode::Certified) (most). Higher = more certification machinery engaged.
    ///
    /// This orders the modes; it is **not** a guarantee strength — a stronger mode never upgrades a
    /// value's [`GuaranteeStrength`](crate::guarantee::GuaranteeStrength) (VR-5). Composing across
    /// modes is an explicit, visible event (RFC-0034 §3.1), never a silent upgrade.
    #[must_use]
    pub fn depth(self) -> u8 {
        match self {
            CertMode::Fast => 0,
            CertMode::Balanced => 1,
            CertMode::Certified => 2,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::CertMode;

    #[test]
    fn default_is_fast() {
        // RFC-0034 §5: `fast` is the project default.
        assert_eq!(CertMode::default(), CertMode::Fast);
    }

    #[test]
    fn depth_orders_fast_balanced_certified() {
        // Strictly increasing certification depth (RFC-0034 §5).
        assert!(CertMode::Fast.depth() < CertMode::Balanced.depth());
        assert!(CertMode::Balanced.depth() < CertMode::Certified.depth());
        // `ALL` is in depth order and exhaustive (the value space is finite — a complete check).
        let depths: Vec<u8> = CertMode::ALL.iter().map(|m| m.depth()).collect();
        assert_eq!(depths, vec![0, 1, 2]);
    }

    #[test]
    fn serde_form_is_the_bare_variant_string() {
        // Mirrors GuaranteeStrength's wire form (RFC-0034 / guarantee.schema.json convention).
        for (mode, json) in [
            (CertMode::Fast, "\"Fast\""),
            (CertMode::Balanced, "\"Balanced\""),
            (CertMode::Certified, "\"Certified\""),
        ] {
            assert_eq!(serde_json::to_string(&mode).unwrap(), json);
            assert_eq!(serde_json::from_str::<CertMode>(json).unwrap(), mode);
        }
    }
}
