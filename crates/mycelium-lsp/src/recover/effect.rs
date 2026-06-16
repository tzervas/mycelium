//! **Declared, bounded effects** (RFC-0014 §4.5) — the safety discipline: effects are *declared* on a
//! signature (no unknown side effects, I3) and any effect that could be unbounded carries an explicit
//! *budget* whose overrun is an **explicit, graceful** [`EffectBudgetExhausted`] (I4) — never a hang, a
//! stack overflow, or an OOM. This is the direct generalisation of the `Fix`/`FixGroup` fuel clock
//! (RFC-0007 §4.5), the M-347 depth ceiling, and DN-05 budgets: **separate named budgets, one
//! enforcement mechanism** (§8 resolved). The default scope is the narrowest — an effect with no budget
//! set cannot run (you opt into a broader effect by *declaring its budget*, I5).
//!
//! v0 is the **reified mechanism + semantics** in the tooling layer; wiring this ledger into the AOT
//! env-machine's runtime budget resolver is the RFC-0008 integration (§4.8 boundary), not v0.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

/// A closed kernel of effect **kinds** (v0; §4.5 I3) plus user-declared names. Coarse by design
/// (KISS/YAGNI) — a declared *set*, not effect-row polymorphism (§9).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EffectKind {
    /// Re-attempting a fallible operation.
    Retry,
    /// Allocating memory.
    Alloc,
    /// Input/output.
    Io,
    /// Triggering a further error/handler (a cascade).
    Cascade,
    /// Consuming a time/fuel-style clock.
    Time,
    /// A downstream user-declared effect (still a name in a known set — never `eval`-ed).
    Named(String),
}

impl fmt::Display for EffectKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EffectKind::Retry => f.write_str("retry"),
            EffectKind::Alloc => f.write_str("alloc"),
            EffectKind::Io => f.write_str("io"),
            EffectKind::Cascade => f.write_str("cascade"),
            EffectKind::Time => f.write_str("time"),
            EffectKind::Named(n) => write!(f, "{n}"),
        }
    }
}

/// A definition's **declared** effect set (§4.5 I3) — what it says it can do, on its signature.
pub type EffectSet = BTreeSet<EffectKind>;

/// An effect a definition performs but did **not** declare (I3) — an explicit checker error, never
/// silent. This is the "no unknown side effects" guarantee.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UndeclaredEffect {
    /// The performed-but-undeclared effect.
    pub effect: EffectKind,
}

impl fmt::Display for UndeclaredEffect {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "undeclared effect {:?}: a definition may not perform an effect absent from its signature \
             (RFC-0014 §4.5 I3 — no unknown side effects); declare it",
            self.effect.to_string()
        )
    }
}

impl std::error::Error for UndeclaredEffect {}

/// The **compositional no-undeclared-effect check** (I3): every effect a definition *performs* (its own
/// plus its callees', composed up the call graph) must be in its *declared* set. Returns the first
/// undeclared effect, if any. This *checks* declared effects compose — it never *infers* one (an effect
/// can never become implicit; §8 resolved: manual-declare + compositional-check).
///
/// # Errors
/// Returns [`UndeclaredEffect`] for the first performed effect not in `declared`.
pub fn check_effects(declared: &EffectSet, performed: &EffectSet) -> Result<(), UndeclaredEffect> {
    for e in performed {
        if !declared.contains(e) {
            return Err(UndeclaredEffect { effect: e.clone() });
        }
    }
    Ok(())
}

/// A per-kind **budget** (§4.5 I4) — distinct vocabulary (`max_attempts` / `max_depth` / a memory
/// ceiling / a fuel clock), all enforced by the one [`Budgets`] mechanism.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectBudget {
    /// Bound on retry attempts.
    Attempts(u64),
    /// Bound on cascade depth.
    Depth(u64),
    /// Bound on bytes allocated.
    Bytes(u64),
    /// Bound on a time/fuel clock.
    Fuel(u64),
}

impl EffectBudget {
    /// The effect kind this budget bounds.
    #[must_use]
    pub fn kind(&self) -> EffectKind {
        match self {
            EffectBudget::Attempts(_) => EffectKind::Retry,
            EffectBudget::Depth(_) => EffectKind::Cascade,
            EffectBudget::Bytes(_) => EffectKind::Alloc,
            EffectBudget::Fuel(_) => EffectKind::Time,
        }
    }
    /// The budget's scalar amount.
    #[must_use]
    pub fn amount(&self) -> u64 {
        match self {
            EffectBudget::Attempts(n)
            | EffectBudget::Depth(n)
            | EffectBudget::Bytes(n)
            | EffectBudget::Fuel(n) => *n,
        }
    }
}

/// Exceeding a budget — an **explicit, graceful** structured error (I4), never a hang / stack overflow
/// / OOM. The analogue of `FuelExhausted` / `DepthLimit` (RFC-0007 §4.5, M-347, DN-05).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectBudgetExhausted {
    /// The effect whose budget was exceeded.
    pub kind: EffectKind,
    /// The amount requested at the overrun.
    pub requested: u64,
    /// The amount remaining when the overrun occurred.
    pub remaining: u64,
}

impl fmt::Display for EffectBudgetExhausted {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "effect budget exhausted for {}: requested {}, {} remaining — an explicit, graceful refusal \
             (RFC-0014 §4.5 I4), never a hang or OOM",
            self.kind, self.requested, self.remaining
        )
    }
}

impl std::error::Error for EffectBudgetExhausted {}

/// The **budget ledger** — one enforcement mechanism over the separate named budgets (§8 resolved).
/// An effect with **no** budget set cannot consume anything (default tightly scoped, I5): you opt into
/// a broader effect by explicitly [`set`](Budgets::set)ting its budget.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Budgets {
    remaining: BTreeMap<EffectKind, u64>,
}

impl Budgets {
    /// An empty ledger — no effect may run until a budget is declared (I5).
    #[must_use]
    pub fn new() -> Self {
        Budgets {
            remaining: BTreeMap::new(),
        }
    }

    /// Builder: declare a budget.
    #[must_use]
    pub fn with(mut self, budget: EffectBudget) -> Self {
        self.set(budget);
        self
    }

    /// Declare (or reset) a budget for its effect kind.
    pub fn set(&mut self, budget: EffectBudget) {
        self.remaining.insert(budget.kind(), budget.amount());
    }

    /// The remaining budget for `kind` (`None` if none was declared).
    #[must_use]
    pub fn remaining(&self, kind: &EffectKind) -> Option<u64> {
        self.remaining.get(kind).copied()
    }

    /// Consume `amount` of `kind`'s budget. An overrun — including consuming an effect with **no**
    /// declared budget — is an explicit, graceful [`EffectBudgetExhausted`] (I4). On success the
    /// remaining budget is decremented.
    ///
    /// # Errors
    /// Returns [`EffectBudgetExhausted`] when `amount` exceeds the remaining budget (or none is set).
    pub fn consume(&mut self, kind: EffectKind, amount: u64) -> Result<(), EffectBudgetExhausted> {
        let remaining = self.remaining.get(&kind).copied().unwrap_or(0);
        if amount > remaining {
            return Err(EffectBudgetExhausted {
                kind,
                requested: amount,
                remaining,
            });
        }
        self.remaining.insert(kind, remaining - amount);
        Ok(())
    }
}
