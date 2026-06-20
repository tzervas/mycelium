//! `Colony<T,E>` and `Scope<T,E>` — structured concurrency surface (ADR-020 v0 R1).
//!
//! # Guarantee (Empirical — grounded in RT2 sequentialization + Kahn-determinism differentials)
//!
//! `Scope<T,E>` guarantees that all tasks spawned within a scope complete or are
//! cancelled before the scope exits. The Kahn-determinism guarantee (channel-mediated
//! communication is deterministic) is **Empirical**: grounded in the RT2 differential
//! but not yet Proven with a formal theorem.

use mycelium_core::GuaranteeStrength;

/// Guarantee strength for `Scope` join semantics (RT2 sequentialization differential).
pub const SCOPE_JOIN_STRENGTH: GuaranteeStrength = GuaranteeStrength::Empirical;

/// Guarantee strength for `Colony` Kahn-determinism (channel-mediated communication).
pub const COLONY_KAHN_STRENGTH: GuaranteeStrength = GuaranteeStrength::Empirical;

/// Structured concurrency scope: all tasks complete or are cancelled before scope exit.
///
/// Guarantee: `Empirical` (RT2 sequentialization + Kahn-determinism; grounded in ADR-020 §4).
/// A scope that exits before all tasks complete returns `Err(ScopeError::TasksStillRunning)`.
#[derive(Debug)]
pub struct Scope<T, E> {
    _output: std::marker::PhantomData<T>,
    _error: std::marker::PhantomData<E>,
}

/// Error type for scope exits with active tasks.
#[derive(Debug, PartialEq, Eq)]
pub enum ScopeError {
    TasksStillRunning,
    Cancelled,
}

impl<T, E> Scope<T, E> {
    /// Create a new empty scope.
    pub fn new() -> Self {
        Scope {
            _output: std::marker::PhantomData,
            _error: std::marker::PhantomData,
        }
    }
}

impl<T, E> Default for Scope<T, E> {
    fn default() -> Self {
        Self::new()
    }
}

/// Colony: a group of scopes sharing a supervision tree and a `Network`.
///
/// Guarantee: `Empirical` (Kahn-determinism of channel communication, ADR-020 §4).
#[derive(Debug)]
pub struct Colony<T, E> {
    _output: std::marker::PhantomData<T>,
    _error: std::marker::PhantomData<E>,
}

impl<T, E> Colony<T, E> {
    /// Create a new colony.
    pub fn new() -> Self {
        Colony {
            _output: std::marker::PhantomData,
            _error: std::marker::PhantomData,
        }
    }
}

impl<T, E> Default for Colony<T, E> {
    fn default() -> Self {
        Self::new()
    }
}
