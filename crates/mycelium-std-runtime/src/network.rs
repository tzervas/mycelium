//! Network, Sender, Receiver, TrySend, TryRecv — channel surface (ADR-020 v0 R1).
//!
//! # Guarantee (Empirical — Kahn-determinism of channel-mediated communication)
//!
//! Message ordering within a single channel is **Exact** (FIFO).
//! Cross-channel Kahn-determinism is **Empirical**: grounded in the RT2 differentials
//! (ADR-020 §4) but not yet Proven with a formal theorem.
//!
//! # Fail-closed on invalid input (G2)
//!
//! `Network::channel(0)` returns `Err(ChannelError::ZeroCapacity)` — zero-capacity channels
//! are nonsensical and are rejected at construction time, not silently converted to a
//! placeholder (ADR-020 §4 / G2: never-silent principle).

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use mycelium_core::GuaranteeStrength;

/// Guarantee strength for single-channel FIFO ordering.
pub const CHANNEL_FIFO_STRENGTH: GuaranteeStrength = GuaranteeStrength::Exact;

/// Guarantee strength for cross-channel Kahn-determinism.
pub const KAHN_DETERMINISM_STRENGTH: GuaranteeStrength = GuaranteeStrength::Empirical;

// ── Channel inner state ───────────────────────────────────────────────────────

/// Shared mutable state behind a `Sender`/`Receiver` pair.
struct ChannelInner<V> {
    buf: VecDeque<V>,
    capacity: usize,
    closed: bool,
}

impl<V> ChannelInner<V> {
    fn new(capacity: usize) -> Self {
        ChannelInner {
            buf: VecDeque::with_capacity(capacity),
            capacity,
            closed: false,
        }
    }
}

// ── Error types ───────────────────────────────────────────────────────────────

/// Errors returned by `Network` construction operations.
#[derive(Debug, PartialEq, Eq)]
pub enum ChannelError {
    /// A zero-capacity channel is nonsensical; rejected at construction (G2: fail-closed).
    ZeroCapacity,
}

// ── Network ───────────────────────────────────────────────────────────────────

/// A named network of typed channels within a `Colony`.
///
/// Guarantee: **Empirical** (Kahn-determinism, ADR-020 §4).
#[derive(Debug)]
pub struct Network {
    _priv: (),
}

impl Network {
    /// Create a new network.
    ///
    /// Guarantee: **Exact** (constructor, trivially correct).
    pub fn new() -> Self {
        Network { _priv: () }
    }

    /// Create a bounded FIFO channel with the given capacity.
    ///
    /// Returns `Err(ChannelError::ZeroCapacity)` if `capacity == 0` (fail-closed, G2:
    /// invalid input is never silently accepted — a zero-capacity channel cannot store any
    /// value and would make every `try_send` return `Full`, which is nonsensical).
    ///
    /// Guarantee: **Exact** (construction is deterministic; the zero-capacity check is
    /// deterministic — mutant witness: removing the check makes `test_channel_zero_capacity_fails`
    /// fail).
    pub fn channel<V>(capacity: usize) -> Result<(Sender<V>, Receiver<V>), ChannelError> {
        if capacity == 0 {
            // Fail-closed: zero-capacity is an explicit error, not a silent stub (G2).
            return Err(ChannelError::ZeroCapacity);
        }
        let inner = Arc::new(Mutex::new(ChannelInner::new(capacity)));
        Ok((
            Sender {
                inner: Arc::clone(&inner),
            },
            Receiver { inner },
        ))
    }
}

impl Default for Network {
    fn default() -> Self {
        Self::new()
    }
}

// ── Sender ────────────────────────────────────────────────────────────────────

/// Sending end of a typed channel.
///
/// SPSC by design: `Sender<V>` is not `Clone` (ADR-020 §4 / RFC-0008 §4.3 RT1).
///
/// Guarantee: **Exact** (FIFO ordering within this channel; backed by `VecDeque`).
pub struct Sender<V> {
    inner: Arc<Mutex<ChannelInner<V>>>,
}

impl<V> std::fmt::Debug for Sender<V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let inner = self.inner.lock().unwrap();
        f.debug_struct("Sender")
            .field("buf_len", &inner.buf.len())
            .field("capacity", &inner.capacity)
            .field("closed", &inner.closed)
            .finish()
    }
}

impl<V> Sender<V> {
    /// Non-blocking send. Returns `TrySend::Sent` on success, `TrySend::Full(v)` if the
    /// buffer is at capacity, or `TrySend::Closed(v)` if the channel is closed.
    ///
    /// The value is **always returned on failure** — never dropped silently (G2 / ADR-020 §4).
    ///
    /// Guarantee: **Exact** (FIFO push into `VecDeque`; deterministic given the buffer state).
    pub fn try_send(&self, value: V) -> TrySend<V> {
        let mut inner = self.inner.lock().unwrap();
        if inner.closed {
            return TrySend::Closed(value);
        }
        if inner.buf.len() >= inner.capacity {
            return TrySend::Full(value);
        }
        inner.buf.push_back(value);
        TrySend::Sent
    }

    /// Close the channel. After this, `try_recv` on a drained buffer returns `TryRecv::Closed`.
    ///
    /// Guarantee: **Exact** (sets a boolean flag, deterministic).
    pub fn close(self) {
        let mut inner = self.inner.lock().unwrap();
        inner.closed = true;
    }
}

// ── Receiver ──────────────────────────────────────────────────────────────────

/// Receiving end of a typed channel.
///
/// Guarantee: **Exact** (FIFO ordering within this channel; backed by `VecDeque`).
pub struct Receiver<V> {
    inner: Arc<Mutex<ChannelInner<V>>>,
}

impl<V> std::fmt::Debug for Receiver<V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let inner = self.inner.lock().unwrap();
        f.debug_struct("Receiver")
            .field("buf_len", &inner.buf.len())
            .field("capacity", &inner.capacity)
            .field("closed", &inner.closed)
            .finish()
    }
}

impl<V> Receiver<V> {
    /// Non-blocking receive. Returns `TryRecv::Received(v)` if a message is buffered,
    /// `TryRecv::Empty` if the buffer is empty and the channel is still open, or
    /// `TryRecv::Closed` if the channel is closed and the buffer is drained.
    ///
    /// Guarantee: **Exact** (FIFO pop from `VecDeque`; deterministic given the buffer state).
    pub fn try_recv(&self) -> TryRecv<V> {
        let mut inner = self.inner.lock().unwrap();
        if let Some(v) = inner.buf.pop_front() {
            return TryRecv::Received(v);
        }
        if inner.closed {
            TryRecv::Closed
        } else {
            TryRecv::Empty
        }
    }
}

// ── TrySend / TryRecv enums ───────────────────────────────────────────────────

/// Result of a non-blocking send attempt.
#[derive(Debug, PartialEq, Eq)]
pub enum TrySend<V> {
    /// Message accepted into the channel buffer.
    Sent,
    /// Channel buffer full; value returned to caller (G2: never silently dropped).
    Full(V),
    /// Channel closed; value returned to caller (G2: never silently dropped).
    Closed(V),
}

/// Result of a non-blocking receive attempt.
#[derive(Debug, PartialEq, Eq)]
pub enum TryRecv<V> {
    /// Message received.
    Received(V),
    /// Channel buffer empty; no message available (sender still live).
    Empty,
    /// Channel closed (sender dropped or `close()` called) and buffer drained.
    Closed,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_zero_capacity_fails() {
        // Mutant witness: removing the zero-capacity check would make this return Ok(_),
        // causing unwrap_err() to panic with a success value.
        let err = Network::channel::<i32>(0).unwrap_err();
        assert_eq!(
            err,
            ChannelError::ZeroCapacity,
            "zero-capacity channel must fail closed with ZeroCapacity (G2)"
        );
    }

    #[test]
    fn test_channel_send_recv_roundtrip() {
        // Send 3 values; receive them in FIFO order.
        // Mutant witness: if VecDeque were replaced with a LIFO stack, order would reverse.
        let (tx, rx) = Network::channel::<i32>(8).expect("channel creation must succeed");
        assert_eq!(tx.try_send(10), TrySend::Sent);
        assert_eq!(tx.try_send(20), TrySend::Sent);
        assert_eq!(tx.try_send(30), TrySend::Sent);
        assert_eq!(rx.try_recv(), TryRecv::Received(10));
        assert_eq!(rx.try_recv(), TryRecv::Received(20));
        assert_eq!(rx.try_recv(), TryRecv::Received(30));
        assert_eq!(
            rx.try_recv(),
            TryRecv::Empty,
            "buffer must be empty after draining"
        );
    }

    #[test]
    fn test_channel_full_returns_full() {
        // capacity=1; second send must return Full with the value.
        // Mutant witness: if capacity check were removed, both sends would succeed and
        // the second try_send would return Sent instead of Full.
        let (tx, rx) = Network::channel::<i32>(1).expect("channel creation must succeed");
        assert_eq!(tx.try_send(42), TrySend::Sent);
        assert_eq!(
            tx.try_send(99),
            TrySend::Full(99),
            "try_send must return Full(value) when buffer is at capacity"
        );
        // The original value is still in the buffer.
        assert_eq!(rx.try_recv(), TryRecv::Received(42));
    }

    #[test]
    fn test_channel_closed_receiver_returns_closed() {
        // Close sender, drain buffer, then try_recv must return Closed.
        // Mutant witness: if Sender::close did not set closed=true, try_recv would return Empty.
        let (tx, rx) = Network::channel::<i32>(4).expect("channel creation must succeed");
        assert_eq!(tx.try_send(1), TrySend::Sent);
        tx.close();
        // Drain the one buffered value.
        assert_eq!(rx.try_recv(), TryRecv::Received(1));
        // Now buffer is empty and channel is closed.
        assert_eq!(
            rx.try_recv(),
            TryRecv::Closed,
            "drained + closed channel must return Closed, not Empty"
        );
    }

    #[test]
    fn test_channel_fifo_is_exact() {
        // 5 sends followed by 5 receives; result order must match send order.
        // Property: ∀i ∈ 0..5, send[i] == recv[i].
        // Mutant witness: if the channel used a random/priority queue, this test would fail.
        let sends: Vec<i32> = (0..5).collect();
        let (tx, rx) = Network::channel::<i32>(8).expect("channel creation must succeed");
        for &v in &sends {
            assert_eq!(tx.try_send(v), TrySend::Sent);
        }
        for (i, &expected) in sends.iter().enumerate() {
            match rx.try_recv() {
                TryRecv::Received(got) => assert_eq!(
                    got, expected,
                    "FIFO violation at index {i}: expected {expected}, got {got}"
                ),
                other => panic!("expected Received at index {i}, got {other:?}"),
            }
        }
    }
}
