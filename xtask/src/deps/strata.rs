//! Loads `xtask/deps-strata.toml` — the frozen stratum map (M-877), the coarse tier map + tier
//! order (M-879 rule `no-upward-tier-edges`), and the named cross-boundary rules (M-879).
//!
//! Compiled in via `include_str!` (production) so the checker and its data file can never drift
//! apart at runtime; tests load their own small synthetic fixtures instead of this file, so the
//! parsing logic here is exercised by both paths.

use std::collections::BTreeMap;

use serde::Deserialize;

/// The full `deps-strata.toml` document.
#[derive(Debug, Deserialize)]
pub struct StrataConfig {
    pub meta: Meta,
    /// crate name -> fine per-crate stratum (M-877).
    pub strata: BTreeMap<String, u32>,
    /// crate name -> coarse tier bucket name (M-879).
    pub tiers: BTreeMap<String, String>,
    /// Declared tier order, lowest (most-depended-upon) first.
    pub tier_order: Vec<String>,
    /// Named, inspectable rules (M-879).
    #[serde(default, rename = "named_rules")]
    pub named_rules: Vec<NamedRule>,
}

#[derive(Debug, Deserialize)]
pub struct Meta {
    pub derived_from: String,
    pub strata_guarantee: String,
    pub tiers_guarantee: String,
    pub basis_ref: String,
}

#[derive(Debug, Deserialize)]
pub struct NamedRule {
    pub id: String,
    pub description: String,
    pub basis_ref: String,
    pub kind: String,
    /// Present only for `kind = "forbidden-target-prefix"`.
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub forbidden_target_prefix: Option<String>,
}

impl StrataConfig {
    /// The production config, embedded at compile time from `xtask/deps-strata.toml`.
    pub fn embedded() -> Self {
        Self::parse(include_str!("../../deps-strata.toml"))
            .expect("xtask/deps-strata.toml must be valid — checked in, not user input")
    }

    /// Parse from a TOML string (used by both `embedded()` and tests, so both paths share one
    /// parser).
    pub fn parse(text: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(text)
    }

    /// The index of `tier` within `tier_order` (lower = more-depended-upon). `None` if `tier`
    /// isn't a declared tier name — never-silent: callers must treat that as a config error, not
    /// default to 0.
    pub fn tier_index(&self, tier: &str) -> Option<usize> {
        self.tier_order.iter().position(|t| t == tier)
    }
}
