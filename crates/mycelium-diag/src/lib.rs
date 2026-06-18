//! `mycelium-diag` — the canonical RFC-0013 structured-diagnostic record types.
//!
//! # Why this crate exists (maintainer decision, 2026-06-18)
//!
//! RFC-0013/RFC-0014 concepts (the structured diagnostic + the recovery bridge) were scattered
//! across `mycelium-check`/`mycelium-l1`/`mycelium-interp`/`mycelium-lsp`. The Phase-5 Tier-A wave
//! (M-510/M-520) needs **one** consolidated reference for the diagnostic record that
//! `std.diag` projects, `std.recover` carries, and `std.testing` records a `Fail` on. Per the
//! maintainer's resolved FLAG (scaffold decision #1), that canonical record is **extracted into
//! this small kernel crate** rather than homed inside `mycelium-std-diag` — a deliberate, bounded
//! growth of the trusted base so the type has a single owner below the std layer. `mycelium-std-diag`
//! re-exports and ergonomically wraps these types (KC-3); it does not redefine them.
//!
//! # Honesty crux (RFC-0013 I1)
//!
//! A `Diag` is **additive over an explicit error**: it presents a failure, it never *is* the
//! failure's control flow. Presentation never gates propagation — there is no severity, note, or
//! locus that makes an underlying error *not* surface. Construction is **total**: a missing locus is
//! [`None`] (explicit), never a fabricated zero (G2).
//!
//! Design spec: `docs/spec/stdlib/diag.md`; RFC-0013; task M-510, issue #151.
//!
//! ## Scaffold status (SCAFFOLD — M-510 leaf to complete)
//!
//! This file defines the **stable record-type contract** (`Diag`, `Severity`, `Locus`, `Trace`,
//! `Code`) as compiling stubs so `mycelium-std-recover` (M-520) can branch from a buildable base.
//! The M-510 leaf agent fills in: the dual human/JSON projection (G11), content-addressed identity
//! (ADR-003), the severity-ordering / never-silent / round-trip property tests, and the §4.5
//! guarantee matrix. Public *signatures* below are the seam other crates depend on — extend, do not
//! break, without FLAGging to the orchestrator.
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

/// Graded diagnostic severity (RFC-0013 §4.1). A **typed** distinction — never a stringly-typed
/// level. Presentation severity **never gates propagation** (I1): a `Warn` never silently becomes a
/// pass, and an `Error` severity does not itself halt anything — it annotates an already-explicit
/// error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Severity {
    /// An error-grade diagnostic.
    Error,
    /// A warning-grade diagnostic.
    Warn,
    /// An informational diagnostic.
    Info,
    /// A debug-grade diagnostic.
    Debug,
}

/// A stable diagnostic code / error class (RFC-0013 §4.2). Closed for the common kernel cases with
/// an explicit [`Code::Other`] escape hatch — never a stringly-typed free-for-all on the common
/// path. The M-510 leaf may widen this set (additively) as the spec's registry is populated.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Code {
    /// A value fell outside its declared range/domain.
    OutOfRange,
    /// A declared, bounded effect budget was exhausted (RFC-0014 I3/I4).
    Budget,
    /// A content-hash / identity mismatch (ADR-003).
    HashMismatch,
    /// An open-coded class identified by a stable string (the registry escape hatch).
    Other(String),
}

/// A source locus — *where* a diagnostic points (RFC-0013 §4.2). All fields are optional: an absent
/// locus stays [`None`] on the [`Diag`], and an absent span/line stays `None` here — **never** a
/// fabricated zero (G2).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct Locus {
    /// Source path/name, if known.
    pub source: Option<String>,
    /// 1-based line, if known.
    pub line: Option<u32>,
    /// 1-based column, if known.
    pub column: Option<u32>,
}

/// An ordered diagnostic trace — the chain of frames/notes that led to the failure (RFC-0013 §4.3).
/// A thin newtype over the frame list so it can grow a richer frame model without breaking callers.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub struct Trace {
    /// Trace frames, outermost-first.
    pub frames: Vec<String>,
}

impl Trace {
    /// The empty trace (explicit absence — not a fabricated frame).
    #[must_use]
    pub fn empty() -> Self {
        Self { frames: Vec::new() }
    }

    /// Push a frame, returning the extended trace (value-semantic).
    #[must_use]
    pub fn with_frame(mut self, frame: impl Into<String>) -> Self {
        self.frames.push(frame.into());
        self
    }
}

/// A structured diagnostic record (RFC-0013 §4.1): a content-addressable value over an
/// already-emitted explicit error. Identity is the record *sans presentation* (ADR-003) — the M-510
/// leaf wires [`Diag::content_hash`]. Builders are total; a missing locus is [`None`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Diag {
    /// The graded severity (typed; never gates propagation — I1).
    pub severity: Severity,
    /// The diagnostic code / error class.
    pub code: Code,
    /// The human-readable message.
    pub message: String,
    /// Where the diagnostic points, if known (explicit `None` when absent).
    pub locus: Option<Locus>,
    /// The diagnostic trace.
    pub trace: Trace,
    /// Free-form notes (EXPLAIN payload, G11).
    pub notes: Vec<String>,
}

impl Diag {
    /// Build an `Error`-severity diagnostic with the given code (total builder).
    #[must_use]
    pub fn error(code: Code) -> Self {
        Self::with_severity(Severity::Error, code)
    }

    /// Build a `Warn`-severity diagnostic with the given code (total builder).
    #[must_use]
    pub fn warn(code: Code) -> Self {
        Self::with_severity(Severity::Warn, code)
    }

    /// Build a `Info`-severity diagnostic with the given code (total builder).
    #[must_use]
    pub fn info(code: Code) -> Self {
        Self::with_severity(Severity::Info, code)
    }

    /// The common total builder behind [`Self::error`]/[`Self::warn`]/[`Self::info`].
    #[must_use]
    pub fn with_severity(severity: Severity, code: Code) -> Self {
        Self {
            severity,
            code,
            message: String::new(),
            locus: None,
            trace: Trace::empty(),
            notes: Vec::new(),
        }
    }

    /// Set the human-readable message (value-semantic builder).
    #[must_use]
    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.message = message.into();
        self
    }

    /// Attach a source locus (explicit; absence stays `None` — never a fabricated zero, G2).
    #[must_use]
    pub fn at(mut self, locus: Locus) -> Self {
        self.locus = Some(locus);
        self
    }

    /// Attach a note (EXPLAIN payload).
    #[must_use]
    pub fn note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }

    /// Replace the trace (value-semantic builder).
    #[must_use]
    pub fn trace(mut self, trace: Trace) -> Self {
        self.trace = trace;
        self
    }

    /// The typed severity (a `Warn` never silently becomes a pass — I1).
    #[must_use]
    pub fn severity(&self) -> Severity {
        self.severity
    }

    /// The diagnostic code / error class.
    #[must_use]
    pub fn code(&self) -> &Code {
        &self.code
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builders_are_total_and_locus_absence_is_explicit() {
        let d = Diag::error(Code::OutOfRange).message("payload len ≠ width");
        assert_eq!(d.severity(), Severity::Error);
        assert_eq!(d.code(), &Code::OutOfRange);
        // A missing locus is explicit None, never a fabricated zero (G2).
        assert!(d.locus.is_none());
    }

    #[test]
    fn at_records_an_explicit_locus() {
        let d = Diag::warn(Code::Budget).at(Locus {
            source: Some("x.myc".into()),
            line: Some(3),
            column: None,
        });
        let l = d.locus.expect("locus set");
        assert_eq!(l.line, Some(3));
        // An absent column stays None — not a fabricated 0.
        assert!(l.column.is_none());
    }
}
