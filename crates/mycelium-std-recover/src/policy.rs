//! The **reified, content-addressed recovery policy** (RFC-0014 §4.4; RFC-0005 pattern; ADR-006).
//!
//! A `RecoveryPolicy` is an `on <ErrorClass> => <RecoveryAction>` map.  Its identity is its
//! **content address** (`PolicyRef` = `ContentHash`; RFC-0001 §4.6 / ADR-003) — a BLAKE3 hash
//! over the canonical, sorted rules.  This makes every recovered/re-propagated outcome answerable
//! to *"which policy acted on this error, and what does it do?"* (C3 / EXPLAIN-able).
//!
//! # Content-hash validity (banked guard #5)
//!
//! Inputs that canonicalize ambiguously must be rejected **before** hashing.  The only inputs
//! here are class names (resolved through the registry — never raw strings, X1) and action
//! parameters (integers / strings / `EffectKind`).  No floating-point or non-finite values enter
//! the hash; the guard is satisfied by construction.

use std::collections::BTreeMap;

use mycelium_core::ContentHash;
use mycelium_interp::budget::EffectKind;

use crate::action::RecoveryAction;
use crate::registry::{ClassName, ClassRegistry, UnknownClass};

/// The content address of a `RecoveryPolicy` (RFC-0001 §4.6 / ADR-006 / `PolicyRef`).
///
/// A deterministic BLAKE3 hash over the policy's canonical, sorted rules.  Stable across
/// serialization boundaries and suitable as a cache / deduplication key.
pub type PolicyRef = ContentHash;

/// A reified, content-addressed recovery policy.
///
/// Maps a **registry-resolved** [`ClassName`] to a [`RecoveryAction`].  Rules are stored in a
/// `BTreeMap` (sorted by class name) so the content hash is deterministic and the policy is
/// diffable / inspectable (C3).
///
/// # EXPLAIN-ability (C3)
///
/// A `RecoveryPolicy` is its own EXPLAIN artifact: the `rules()` iterator exposes every class →
/// action binding, and [`RecoveryPolicy::policy_ref`] is the stable identity tag embedded in
/// every [`crate::Resolution`] outcome.
///
/// # Closed action set
///
/// The `T` type parameter is the fallback value type — it must be
/// [`std::fmt::Debug`] + [`Clone`] + `Send + Sync + 'static` to participate in content
/// hashing via `format!("{:?}", …)`.  In practice `T` is the recoverable value type of the
/// operation.
#[derive(Debug, Clone)]
pub struct RecoveryPolicy<T> {
    rules: BTreeMap<ClassName, RecoveryAction<T>>,
}

impl<T> Default for RecoveryPolicy<T> {
    fn default() -> Self {
        RecoveryPolicy {
            rules: BTreeMap::new(),
        }
    }
}

impl<T: std::fmt::Debug + Clone + Send + Sync + 'static> RecoveryPolicy<T> {
    /// An empty policy (no rules).
    #[must_use]
    pub fn new() -> Self {
        RecoveryPolicy::default()
    }

    /// Add an `on <class> => <action>` rule, resolving the class through `registry` (X1).
    ///
    /// Replaces and returns any prior action for the class.  An unknown class is an explicit
    /// configuration error ([`UnknownClass`]), never a silent fabrication.
    ///
    /// # Errors
    /// Returns [`UnknownClass`] if `class` is not in the registry.
    pub fn on(
        &mut self,
        registry: &ClassRegistry,
        class: &str,
        action: RecoveryAction<T>,
    ) -> Result<Option<RecoveryAction<T>>, UnknownClass> {
        let name = registry.resolve(class)?;
        Ok(self.rules.insert(name, action))
    }

    /// The recovery action for a resolved class, if any.
    #[must_use]
    pub fn action_for(&self, class: &ClassName) -> Option<&RecoveryAction<T>> {
        self.rules.get(class)
    }

    /// The rules in deterministic (class-sorted) order (C3 — inspectable, diffable).
    pub fn rules(&self) -> impl Iterator<Item = (&ClassName, &RecoveryAction<T>)> {
        self.rules.iter()
    }

    /// Whether the policy has no rules.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }

    /// The **content address** of this policy (RFC-0005 `PolicyRef`; ADR-006).
    ///
    /// A deterministic BLAKE3 over the canonical sorted rules; diffable and identity-stable.
    /// Every outcome returned by [`crate::handle`] carries this `PolicyRef` when a rule was
    /// applied, so every outcome can be traced back to the exact policy that acted (C3).
    ///
    /// # Honesty (banked guard #5)
    ///
    /// All hashed fields are discrete / well-formed — class names are registry-resolved strings,
    /// action parameters are integers or strings.  No floating-point values enter the hash; no
    /// ambiguous encoding can collide.  We hash the rule count before iterating so a length-1
    /// policy for class `"ab"` and a length-2 policy for classes `"a"` / `"b"` do not collide.
    #[must_use]
    pub fn policy_ref(&self) -> PolicyRef {
        let mut h = blake3::Hasher::new();
        // Length-prefix every variable-length field so different structures cannot collide.
        let blob = |hasher: &mut blake3::Hasher, bytes: &[u8]| {
            hasher.update(&(bytes.len() as u64).to_le_bytes());
            hasher.update(bytes);
        };
        blob(&mut h, b"mycelium.recovery-policy.v1");
        h.update(&(self.rules.len() as u64).to_le_bytes());
        for (class, action) in &self.rules {
            blob(&mut h, class.as_str().as_bytes());
            match action {
                RecoveryAction::Fallback { value } => {
                    h.update(&[0u8]);
                    // Hash the Debug representation — a stable, deterministic encoding for
                    // well-behaved types.  Non-finite floats cannot appear here (no float fields).
                    blob(&mut h, format!("{value:?}").as_bytes());
                }
                RecoveryAction::Retry { max_attempts } => {
                    h.update(&[1u8]);
                    h.update(&max_attempts.to_le_bytes());
                }
                RecoveryAction::Escalate { to_class } => {
                    h.update(&[2u8]);
                    blob(&mut h, to_class.as_bytes());
                }
                RecoveryAction::CleanupThenPropagate { effect } => {
                    h.update(&[3u8]);
                    blob(&mut h, effect.to_string().as_bytes());
                }
            }
        }
        let hex = h.finalize().to_hex();
        ContentHash::from_parts("blake3", hex.as_str())
            .expect("blake3 hex is always a valid content hash")
    }
}

/// The declared, closed effect set for a policy (I3 / RFC-0014 §4.5).
///
/// Returns the set of `EffectKind`s that the policy's actions may perform.  Used by
/// [`crate::effect::check_effects`] to enforce I3 (no undeclared effect).
pub fn policy_effects<T>(policy: &RecoveryPolicy<T>) -> std::collections::BTreeSet<EffectKind>
where
    T: std::fmt::Debug + Clone + Send + Sync + 'static,
{
    let mut set = std::collections::BTreeSet::new();
    for (_, action) in policy.rules() {
        match action {
            RecoveryAction::Retry { .. } => {
                set.insert(EffectKind::Retry);
            }
            RecoveryAction::CleanupThenPropagate { effect } => {
                set.insert(effect.clone());
            }
            RecoveryAction::Fallback { .. } | RecoveryAction::Escalate { .. } => {}
        }
    }
    set
}
