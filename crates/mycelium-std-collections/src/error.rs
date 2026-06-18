//! Explicit error types for `std.collections` (C1 — never-silent; RFC-0016 §4.1).
//!
//! Every fallible operation in this crate returns a typed error or `Option`, never a
//! sentinel or silently-clamped value. [`CollErr`] is the only error variant — it covers
//! the out-of-bounds / bad slice-range cases (spec §3).

use std::fmt;

/// Out-of-bounds or invalid range on a [`crate::Seq`] operation (spec §3 `CollErr`).
///
/// # C1 compliance
/// [`crate::seq::Seq::update`] and [`crate::seq::Seq::slice`] return `Err(CollErr::IndexOOB)`
/// rather than silently clamping or returning a sentinel value (RFC-0016 §4.1 C1; G2
/// "never-silent"). The `context` field carries a human-readable description of *which bound
/// was violated*, enabling the caller to surface the diagnostic without stripping information
/// (G11 dual projection; RFC-0013 structured diagnostic).
///
/// # EXPLAIN (C3)
/// `IndexOOB` is the reified *refusal record* for `update` / `slice` (spec §4 "the refusal
/// record"). The fields are inspectable: a caller can see `index`, `len`, and `context`
/// without parsing strings.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CollErr {
    /// An index exceeded the valid range, or a slice range was invalid.
    ///
    /// # Fields
    /// - `index` — the offending index (or `lo` for a slice).
    /// - `len` — the length of the collection at the time of the operation.
    /// - `context` — a static description of which constraint was violated (G11).
    IndexOOB {
        /// The index (or slice bound) that was out of range.
        index: usize,
        /// The length of the collection at the time of the operation.
        len: usize,
        /// Which constraint was violated (e.g. `"i >= len"`, `"lo > hi"`, `"hi > len"`).
        context: &'static str,
    },
}

impl CollErr {
    /// Construct an `IndexOOB` error.
    pub(crate) fn index_oob(index: usize, len: usize, context: &'static str) -> Self {
        CollErr::IndexOOB {
            index,
            len,
            context,
        }
    }
}

impl fmt::Display for CollErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CollErr::IndexOOB {
                index,
                len,
                context,
            } => write!(
                f,
                "index out of bounds: {context} (index={index}, len={len})"
            ),
        }
    }
}

impl std::error::Error for CollErr {}

#[cfg(test)]
mod tests {
    use super::CollErr;

    #[test]
    fn index_oob_display_includes_all_fields() {
        // Guard: mutation of index, len, or context makes this fail.
        let e = CollErr::index_oob(5, 3, "i >= len");
        let s = e.to_string();
        assert!(s.contains('5'), "display must include index");
        assert!(s.contains('3'), "display must include len");
        assert!(s.contains("i >= len"), "display must include context");
    }

    #[test]
    fn coll_err_is_an_std_error() {
        // Compile-time check: it satisfies the std::error::Error bound.
        let e = CollErr::index_oob(0, 0, "i >= len");
        let _: &dyn std::error::Error = &e;
    }
}
