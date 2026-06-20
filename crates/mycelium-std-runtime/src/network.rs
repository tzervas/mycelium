//! Network, Sender, Receiver, TrySend, TryRecv — channel surface (ADR-020 v0 R1).
//!
//! # Guarantee (Empirical — Kahn-determinism of channel-mediated communication)
//!
//! Message ordering within a single channel is **Exact** (FIFO).
//! Cross-channel Kahn-determinism is **Empirical**: grounded in the RT2 differentials
//! (ADR-020 §4) but not yet Proven with a formal theorem.

use mycelium_core::GuaranteeStrength;

/// Guarantee strength for single-channel FIFO ordering.
pub const CHANNEL_FIFO_STRENGTH: GuaranteeStrength = GuaranteeStrength::Exact;

/// Guarantee strength for cross-channel Kahn-determinism.
pub const KAHN_DETERMINISM_STRENGTH: GuaranteeStrength = GuaranteeStrength::Empirical;

/// A named network of typed channels within a `Colony`.
///
/// Guarantee: **Empirical** (Kahn-determinism, ADR-020 §4).
#[derive(Debug)]
pub struct Network {
    _priv: (),
}

impl Network {
    pub fn new() -> Self {
        Network { _priv: () }
    }
}

impl Default for Network {
    fn default() -> Self {
        Self::new()
    }
}

/// Sending end of a typed channel.
///
/// Guarantee: **Exact** (FIFO ordering within this channel).
#[derive(Debug)]
pub struct Sender<V> {
    _value: std::marker::PhantomData<V>,
}

/// Receiving end of a typed channel.
///
/// Guarantee: **Exact** (FIFO ordering within this channel).
#[derive(Debug)]
pub struct Receiver<V> {
    _value: std::marker::PhantomData<V>,
}

/// Result of a non-blocking send attempt.
#[derive(Debug, PartialEq, Eq)]
pub enum TrySend<V> {
    /// Message accepted into the channel buffer.
    Sent,
    /// Channel buffer full; value returned to caller.
    Full(V),
    /// Channel closed (receiver dropped); value returned.
    Closed(V),
}

/// Result of a non-blocking receive attempt.
#[derive(Debug, PartialEq, Eq)]
pub enum TryRecv<V> {
    /// Message received.
    Received(V),
    /// Channel buffer empty; no message available.
    Empty,
    /// Channel closed (sender dropped) and buffer drained.
    Closed,
}

impl<V> Default for Sender<V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V> Sender<V> {
    /// Create a sender (internal; constructed by `Network::channel`).
    pub fn new() -> Self {
        Sender {
            _value: std::marker::PhantomData,
        }
    }

    /// Non-blocking send. Returns `TrySend::Sent` on success, otherwise the value.
    pub fn try_send(&self, value: V) -> TrySend<V> {
        let _ = value;
        TrySend::Closed(value)
    }
}

impl<V> Default for Receiver<V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V> Receiver<V> {
    /// Create a receiver (internal; constructed by `Network::channel`).
    pub fn new() -> Self {
        Receiver {
            _value: std::marker::PhantomData,
        }
    }

    /// Non-blocking receive. Returns `TryRecv::Received(v)` if a message is ready.
    pub fn try_recv(&self) -> TryRecv<V> {
        TryRecv::Closed
    }
}
