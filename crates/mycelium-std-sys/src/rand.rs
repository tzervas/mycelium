//! \[Declared\] Platform entropy floor. v0 fills a buffer with `DefaultHasher`+`SystemTime` bytes.
//!
//! **Not OS entropy.** See the implementation note below — the v0 stand-in is a hash of the
//! current system time, not a call to `getrandom` or `/dev/urandom`.
//! Declared — no audit of RNG quality; wiring from `std-rand` deferred to a future wave.
//!
//! # Note on implementation
//!
//! v0 uses `std::collections::hash_map::DefaultHasher` seeded from `SystemTime` as a best-effort
//! stand-in for a real OS entropy source. A production implementation would use `getrandom` or
//! `/dev/urandom` directly (requiring a dependency or an `unsafe` FFI block). This establishes
//! the interface; the real entropy plumbing is the next wave's work.
//!
//! `Declared`: this is NOT a cryptographically vetted entropy source. Do not use for
//! security-sensitive purposes until the implementation is upgraded and retagged.

use std::fmt;

/// Errors from platform entropy operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntropyError {
    /// The platform entropy source is unavailable or exhausted.
    Unavailable(String),
}

impl fmt::Display for EntropyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EntropyError::Unavailable(msg) => write!(f, "entropy unavailable: {msg}"),
        }
    }
}

impl std::error::Error for EntropyError {}

/// \[Declared\] Fill `buf` with bytes from the platform entropy source.
///
/// Never-silent: returns `Err(EntropyError::Unavailable(...))` on any failure; never panics.
///
/// # Guarantee
///
/// `Declared` — v0 uses a `DefaultHasher`+`SystemTime` stand-in. No cryptographic audit.
/// A real OS entropy source (`getrandom` / `/dev/urandom`) is the next-wave upgrade target.
pub fn fill_bytes(buf: &mut [u8]) -> Result<(), EntropyError> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::time::SystemTime;

    let seed = SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| EntropyError::Unavailable(e.to_string()))?
        .as_nanos();

    let mut h = DefaultHasher::new();
    for (i, byte) in buf.iter_mut().enumerate() {
        (seed ^ (i as u128)).hash(&mut h);
        *byte = (h.finish() & 0xFF) as u8;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fill_bytes_32_succeeds() {
        let mut buf = [0u8; 32];
        let result = fill_bytes(&mut buf);
        assert!(result.is_ok(), "fill_bytes returned Err: {result:?}");
        // We can't assert randomness, but a Declared best-effort implementation
        // should produce at least *some* non-zero bytes for a 32-byte buffer.
        // (This is a basic smoke test, not a statistical test.)
    }

    #[test]
    fn fill_bytes_empty_ok() {
        let mut buf = [];
        let result = fill_bytes(&mut buf);
        assert!(result.is_ok(), "fill_bytes([]) returned Err: {result:?}");
    }

    #[test]
    fn entropy_error_display() {
        let e = EntropyError::Unavailable("test reason".to_string());
        assert_eq!(e.to_string(), "entropy unavailable: test reason");
    }
}
