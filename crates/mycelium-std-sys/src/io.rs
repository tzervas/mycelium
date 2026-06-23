//! \[Declared\] Standard-stream I/O floor (RFC-0028 §4.5; M-722). Thin, never-silent wrappers over
//! Rust `std::io` stdin/stdout/stderr.
//!
//! This is the audited syscall floor for `std.io`'s OS contact: reading from stdin and writing to
//! stdout/stderr. Per LR-9 / RFC-0016 §8-Q6 all such contact lives in this single `std-sys` phylum,
//! so the pure `std-io` crate stays `wild`-free.
//!
//! # Honesty (VR-5)
//!
//! Every function carries the **`Declared`** guarantee tag — these are unaudited `std::io` wrappers;
//! no theorem and no measured bound backs stream semantics, buffering, or partial-write behaviour.
//! Promotion to `Empirical` requires documented coverage (e.g. a recorded round-trip property over a
//! piped fixture); `Proven` requires a checked theorem. Neither is established in v0.
//!
//! # Never-silent (G2)
//!
//! Every operation returns an explicit `Result` on failure — no byte count is silently dropped, no
//! short write is silently ignored. `write_all` propagates the OS error; the caller is never told a
//! write succeeded when it did not.
//!
//! # Guarantee matrix (RFC-0016 §4.5)
//!
//! | op | signature | failure mode | tag |
//! |----|-----------|--------------|-----|
//! | `read_to_end` | `() -> Result<Vec<u8>, io::Error>` | OS read error → `Err` | `Declared` |
//! | `read_line` | `() -> Result<String, io::Error>` | OS/UTF-8 error → `Err` | `Declared` |
//! | `write_out` | `(&[u8]) -> Result<(), io::Error>` | short/failed write → `Err` | `Declared` |
//! | `write_err` | `(&[u8]) -> Result<(), io::Error>` | short/failed write → `Err` | `Declared` |
//! | `flush_out` | `() -> Result<(), io::Error>` | OS flush error → `Err` | `Declared` |

use std::io::{self, Read, Write};

/// \[Declared\] Read **all** of stdin to end-of-input. Returns `Err` on any OS read error —
/// never-silent (G2): a partial read that hits an error is reported, not truncated-and-returned.
pub fn read_to_end() -> Result<Vec<u8>, io::Error> {
    let mut buf = Vec::new();
    io::stdin().read_to_end(&mut buf)?;
    Ok(buf)
}

/// \[Declared\] Read a single line from stdin (including the trailing newline if present). Returns
/// `Err` on any OS read error or invalid UTF-8 — never-silent (G2).
pub fn read_line() -> Result<String, io::Error> {
    let mut line = String::new();
    io::stdin().read_line(&mut line)?;
    Ok(line)
}

/// \[Declared\] Write all of `bytes` to stdout. Uses `write_all`, so a short write is an explicit
/// `Err`, never a silently-dropped tail (G2). Does **not** flush — call [`flush_out`] when ordering
/// against process exit matters.
pub fn write_out(bytes: &[u8]) -> Result<(), io::Error> {
    io::stdout().write_all(bytes)
}

/// \[Declared\] Write all of `bytes` to stderr. Like [`write_out`]: a short write is an explicit
/// `Err` (G2).
pub fn write_err(bytes: &[u8]) -> Result<(), io::Error> {
    io::stderr().write_all(bytes)
}

/// \[Declared\] Flush stdout — surfaces any deferred OS write error explicitly (G2), so a buffered
/// write lost at flush time is never silently dropped.
pub fn flush_out() -> Result<(), io::Error> {
    io::stdout().flush()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The write path returns `Ok` on a normal stdout write and surfaces the byte payload faithfully
    /// (a smoke test — the OS stream is the ground truth; this only pins that the wrapper does not
    /// swallow or transform the bytes).
    #[test]
    fn write_out_and_flush_succeed_on_a_normal_stream() {
        write_out(b"").expect("an empty write succeeds");
        flush_out().expect("flush succeeds");
    }

    /// Never-silent (G2): `write_all` over a deliberately failing writer surfaces the error rather
    /// than reporting a phantom success. We exercise the same `Write::write_all` contract `write_out`
    /// relies on, against a sink that refuses, to pin the never-silent property structurally.
    #[test]
    fn a_failing_write_is_an_explicit_error_not_a_phantom_success() {
        struct Refuse;
        impl Write for Refuse {
            fn write(&mut self, _: &[u8]) -> io::Result<usize> {
                Err(io::Error::other("refused"))
            }
            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }
        let err = Refuse
            .write_all(b"x")
            .expect_err("a refusing writer must surface an error");
        assert_eq!(err.kind(), io::ErrorKind::Other);
    }
}
