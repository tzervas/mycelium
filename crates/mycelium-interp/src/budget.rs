//! **Effect budgets** — the shared budget-resolution surface for *declared, bounded effects*
//! (RFC-0014 §4.5/§4.8), unified with the runtime's existing fuel/depth clocks (RFC-0007 §4.5,
//! M-347, DN-05).
//!
//! This module is the **one home** the RFC-0014 §4.8 integration required: the recovery `Budgets`
//! ledger (formerly tooling-only, in `mycelium-lsp/src/recover/effect.rs`) and the AOT env-machine
//! (`mycelium-mlir`) both depend on `mycelium-interp`, with no edge between them — so the ledger
//! primitive lives here, alongside the fuel clock and [`EvalError`](crate::EvalError). An effect
//! overrun is routed through [`EvalError::EffectBudget`](crate::EvalError::EffectBudget): **one
//! enforcement mechanism over separate named budgets** (RFC-0014 §8 disposition) — a budgeted effect
//! overruns *gracefully* at runtime exactly as a runaway recursion hits [`FuelExhausted`] /
//! [`DepthLimit`], never a hang or an OOM.
//!
//! It introduces **no L0 node** (KC-3): these are runtime/checker types, not kernel calculus — the
//! ledger lives where fuel/depth already live, mirroring how the totality checker lives outside the
//! trusted base (RFC-0014 §8, maintainer 2026-06-16: no kernel-visible hook).
//!
//! [`FuelExhausted`]: crate::EvalError::FuelExhausted
//! [`DepthLimit`]: crate::EvalError::DepthLimit

use std::collections::BTreeMap;
use std::fmt;

/// A closed kernel of effect **kinds** (RFC-0014 §4.5 I3) plus user-declared names. Coarse by design
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

/// A per-kind **budget** (RFC-0014 §4.5 I4) — distinct vocabulary (`max_attempts` / `max_depth` / a
/// memory ceiling / a fuel clock), all enforced by the one [`Budgets`] mechanism.
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

/// Exceeding a budget — an **explicit, graceful** structured error (RFC-0014 §4.5 I4), never a hang /
/// stack overflow / OOM. The effect analogue of [`FuelExhausted`] / [`DepthLimit`] (RFC-0007 §4.5,
/// M-347, DN-05); converts into [`EvalError::EffectBudget`] so the *runtime* refuses an effect overrun
/// through the same explicit channel as recursion.
///
/// [`FuelExhausted`]: crate::EvalError::FuelExhausted
/// [`DepthLimit`]: crate::EvalError::DepthLimit
/// [`EvalError::EffectBudget`]: crate::EvalError::EffectBudget
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

/// The **budget ledger** — one enforcement mechanism over the separate named budgets (RFC-0014 §8
/// resolved). An effect with **no** budget set cannot consume anything (default tightly scoped, I5):
/// you opt into a broader effect by explicitly [`set`](Budgets::set)ting its budget.
///
/// The env-machine threads a `&mut Budgets` (alongside its fuel/depth clocks) and the recovery driver
/// consumes the *same* type, so an overrun surfaces as [`EvalError::EffectBudget`] on the one runtime
/// refusal path (RFC-0014 §4.8). The concurrency wave (RFC-0008) layers *per-task* ledgers on this seam.
///
/// [`EvalError::EffectBudget`]: crate::EvalError::EffectBudget
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EvalError;

    #[test]
    fn an_undeclared_budget_refuses_immediately() {
        // Default tightly scoped (I5): an effect with no declared budget cannot run.
        let mut b = Budgets::new();
        let err = b.consume(EffectKind::Retry, 1).unwrap_err();
        assert_eq!(err.kind, EffectKind::Retry);
        assert_eq!(err.remaining, 0);
    }

    #[test]
    fn a_declared_budget_drains_then_overruns_explicitly() {
        let mut b = Budgets::new().with(EffectBudget::Attempts(2));
        assert!(b.consume(EffectKind::Retry, 1).is_ok());
        assert_eq!(b.remaining(&EffectKind::Retry), Some(1));
        assert!(b.consume(EffectKind::Retry, 1).is_ok());
        // The overrun is the explicit, graceful refusal — never a hang.
        let err = b.consume(EffectKind::Retry, 1).unwrap_err();
        assert_eq!(err.requested, 1);
        assert_eq!(err.remaining, 0);
    }

    #[test]
    fn an_overrun_routes_through_the_runtime_eval_error_channel() {
        // RFC-0014 §4.8: an effect-budget overrun is a *runtime* refusal on the same channel as
        // FuelExhausted/DepthLimit — one enforcement mechanism over separate named budgets.
        let mut b = Budgets::new().with(EffectBudget::Depth(0));
        let exhausted = b.consume(EffectKind::Cascade, 1).unwrap_err();
        let as_eval: EvalError = exhausted.clone().into();
        assert_eq!(as_eval, EvalError::EffectBudget(exhausted));
    }
}
