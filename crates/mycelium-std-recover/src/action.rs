//! The **closed** v0 recovery-action set (RFC-0014 §4.4; §8 resolved closed).
//!
//! Each action yields an explicit outcome — a recovered value or a re-propagated error — and
//! there is no action that silently discards an error (I1). The set is **closed** in v0 (spec §8
//! resolved); user-defined compound actions are a §9 future.

use mycelium_interp::budget::EffectKind;

/// The **closed** v0 recovery-action set (RFC-0014 §4.4; §8 resolved).
///
/// Each variant is an explicit instruction to the [`crate::handle`] driver; together they cover
/// the four recovery archetypes: substitute a value (`fallback`), re-attempt the operation
/// (`retry`), redirect the error class (`escalate`), run a bounded cleanup and propagate
/// (`cleanup_then_propagate`).
///
/// # Guarantee tags (I2/VR-5 — recovery only ever downgrades)
///
/// - [`RecoveryAction::Fallback`]: the substituted value is honestly tagged **`Declared`**; a
///   fallback has no checked basis and can be at most `Declared`.
/// - [`RecoveryAction::Retry`]: inherits the successful attempt's **own** tag; on exhaustion no
///   value is produced and the original error propagates.
/// - [`RecoveryAction::Escalate`]: re-propagates — no recovered value, so no guarantee tag
///   question (the error's class is transformed, its identity is preserved).
/// - [`RecoveryAction::CleanupThenPropagate`]: re-propagates the original error — same as above.
///
/// # Never-silent (I1)
///
/// No action variant makes an error vanish. `Fallback` replaces it with an explicit value;
/// `Retry` propagates the original on exhaustion; `Escalate` and `CleanupThenPropagate`
/// re-propagate an explicit error. A handler using this closed set **cannot** express a "drop"
/// — the type enforces I1.
#[derive(Debug, Clone, PartialEq)]
pub enum RecoveryAction<T> {
    /// Recover with an explicit fallback value.
    ///
    /// The recovered value is honestly tagged **`Declared`** (a substituted fallback has no
    /// checked basis — I2/VR-5). This is the **only** action with a fixed recovered tag; every
    /// other action either inherits its tag from the attempt or does not produce a value.
    ///
    /// # Never-silent (I1)
    /// Always yields `Recovered(value, Declared)` — never a drop.
    ///
    /// # Effects
    /// None — a pure value substitution.
    Fallback {
        /// The fallback value (boxed to flatten the action enum's discriminant size).
        value: Box<T>,
    },
    /// Re-attempt the operation, bounded by `max_attempts` (I4).
    ///
    /// On each attempt the driver calls the provided `attempt` thunk. If an attempt succeeds its
    /// value is recovered with its **own** honest guarantee (inherited, never upgraded — I2). If
    /// **all** `max_attempts` fail the **original** error continues to propagate (additive — I1).
    ///
    /// # Never-silent (I1)
    /// Either `Recovered` (from a successful attempt) or `Propagated(original_error)` on
    /// exhaustion. The original error is never discarded.
    ///
    /// # Effects
    /// Declares **`EffectKind::Retry`**, budgeted `Attempts(max_attempts)` (I4). A budget
    /// overrun is an explicit [`mycelium_interp::budget::EffectBudgetExhausted`], never a hang.
    Retry {
        /// The maximum number of re-attempts (the budget ceiling, I4; must be ≥ 1).
        max_attempts: u64,
    },
    /// Transform and re-propagate as a different error class — still explicit.
    ///
    /// The error is re-tagged with `to_class` (a string naming a registry-resolved class —
    /// X1: the class is a name, never an evaluated string). It always re-propagates; there is no
    /// recovered value.
    ///
    /// # Never-silent (I1)
    /// Always `Propagated(transformed_error)` — the error's existence is never hidden.
    ///
    /// # Effects
    /// None (a pure structural transform — the class label changes, the error continues).
    Escalate {
        /// The class to escalate into (registry-resolved name; X1).
        to_class: String,
    },
    /// Run a **bounded** effect then let the original error continue (additive).
    ///
    /// The driver consumes `effect` from the ambient budget ledger (one enforcement mechanism —
    /// RFC-0014 §4.8 / `mycelium-interp`). A budget overrun skips the cleanup **only** — the
    /// original error propagates regardless (I1). The cleanup's budget overrun is noted in the
    /// returned [`crate::Resolution::Propagated`] as an optional `cleanup_overrun` flag so the
    /// failure is legible, not silently swallowed (spec §7-Q4 disposition: record it).
    ///
    /// # Never-silent (I1)
    /// The original error always propagates, whether the cleanup succeeds or overruns.
    ///
    /// # Effects
    /// Declares `effect`, budgeted in the [`mycelium_interp::budget::Budgets`] ledger (I4).
    CleanupThenPropagate {
        /// The declared cleanup effect (its budget is in the ambient ledger; I5).
        effect: EffectKind,
    },
}
