//! Guarantee matrix for `std.testing` (M-534, #174) — RFC-0016 §4.5.
//!
//! Encoded as checked data + asserted in tests (never prose only — spec §4 / README §2).
//!
//! # Tag justification (VR-5 — downgrade rather than overclaim)
//! The harness ops are **`Exact` mechanisms**: a `Verdict` is an exact, deterministic function
//! of the run (seeded property → reproducible; golden compare → exact equality; differential →
//! exact equality of observables). This `Exact` is about the *verdict mechanism*, **not** a
//! claim about the subject under test.
//!
//! The harness **never inflates the subject's tag** (C2 crux): a passing `for_all` backs
//! `Empirical`, not `Proven`. There is no operation in this module that turns
//! "passed N trials" into `Proven` — that would be the VR-5 violation the module exists to
//! prevent (spec §4 tag justification).

use mycelium_core::GuaranteeStrength;

/// One row of the `std.testing` guarantee matrix.
///
/// Mirrors the `GuaranteeRow` shape from `mycelium-std-core` (spec §4 / README §2) — kept
/// local to avoid a circular crate dependency while `std.core` is not yet a dep.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Row {
    /// The exported op name.
    pub op: &'static str,
    /// Guarantee tag on the `Exact ⊐ Proven ⊐ Empirical ⊐ Declared` lattice (RFC-0001 §4.3).
    pub tag: GuaranteeStrength,
    /// Explicit fallibility: `"total"` or the `Option`/`Result`/`Verdict` shape returned.
    pub fallibility: &'static str,
    /// Declared effects (`"none"` for pure ops; `"io (baseline read)"` for golden).
    pub effects: &'static str,
    /// Whether the op surfaces an EXPLAIN artifact (C3/SC-3/G11).
    pub explainable: bool,
}

/// The `std.testing` guarantee matrix (spec §4).
///
/// Every row is `Exact` (the verdict mechanism is exact) and either effect-free or declares
/// its IO effect explicitly (C6). EXPLAIN coverage is on every harness op that produces a
/// non-`Pass` verdict (C3).
///
/// | Op | Tag | Fallibility | Effects | EXPLAIN |
/// |---|---|---|---|---|
/// | `for_all` | Exact | `Fail{Diag}` / `Skipped{...}` as Verdict | none (pure; seeded) | yes |
/// | `golden` | Exact | `Fail{diff}` / `Skipped{NeedsRecord}` | io (baseline read) | yes |
/// | `differential` | Exact | `Fail{lhs,rhs}` / `Skipped{BackendUnavailable}` | none / io per backend | yes |
/// | `summarize` | Exact | total | none | yes |
/// | `is_green` | Exact | total | none | yes |
pub const MATRIX: &[Row] = &[
    Row {
        op: "for_all",
        tag: GuaranteeStrength::Exact,
        fallibility: "Verdict::Fail{record} / Verdict::Skipped{NeedsRecord}",
        effects: "none (pure; seeded — C6/RT3)",
        explainable: true, // counterexample + seed (C3/G11)
    },
    Row {
        op: "golden",
        tag: GuaranteeStrength::Exact,
        fallibility: "Verdict::Fail{diff} / Verdict::Skipped{NeedsRecord}",
        effects: "io (baseline read — declared, C6)",
        explainable: true, // diff + baseline hash (C3/G11)
    },
    Row {
        op: "differential",
        tag: GuaranteeStrength::Exact,
        fallibility: "Verdict::Fail{lhs,rhs} / Verdict::Skipped{BackendUnavailable}",
        effects: "none / io per backend (declared, C6)",
        explainable: true, // both outputs + input (C3/G11)
    },
    Row {
        op: "summarize",
        tag: GuaranteeStrength::Exact,
        fallibility: "total",
        effects: "none",
        explainable: true, // per-class counts (C3)
    },
    Row {
        op: "is_green",
        tag: GuaranteeStrength::Exact,
        fallibility: "total",
        effects: "none",
        explainable: true, // caller can inspect Summary for skip/undetermined counts (C3)
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    /// The matrix has exactly 5 rows (spec §4 lists five ops).
    #[test]
    fn matrix_has_five_rows() {
        assert_eq!(
            MATRIX.len(),
            5,
            "spec §4 lists five ops in the guarantee matrix"
        );
    }

    /// Every row is `Exact` (spec §4 tag justification: harness ops are Exact mechanisms).
    /// Guard: accidentally tagging any row Empirical/Declared would overclaim subject strength.
    #[test]
    fn all_rows_are_exact() {
        for row in MATRIX {
            assert_eq!(
                row.tag,
                GuaranteeStrength::Exact,
                "{} must be Exact — harness ops are Exact mechanisms (spec §4 / VR-5)",
                row.op
            );
        }
    }

    /// All five rows are EXPLAIN-able (the harness ops all surface inspection artifacts — C3).
    #[test]
    fn all_rows_are_explainable() {
        for row in MATRIX {
            assert!(
                row.explainable,
                "{} must be EXPLAIN-able (C3/G11/SC-3 — no black boxes)",
                row.op
            );
        }
    }

    /// Op names are unique (no duplicate rows).
    #[test]
    fn op_names_are_unique() {
        let mut names: std::collections::HashSet<&str> = std::collections::HashSet::new();
        for row in MATRIX {
            assert!(
                names.insert(row.op),
                "duplicate op name '{}' in guarantee matrix",
                row.op
            );
        }
    }

    /// The harness ops that declare IO effects are `golden` and `differential` only.
    /// (Spec §4: `golden` declares `io (read baseline)`; `differential` declares `io per backend`.)
    /// Guard: accidentally marking effect-free ops as IO-declaring makes this fail.
    #[test]
    fn only_golden_and_differential_declare_io() {
        for row in MATRIX {
            if row.op == "golden" || row.op == "differential" {
                assert!(
                    row.effects.contains("io"),
                    "{} must declare IO effects (spec §4)",
                    row.op
                );
            } else {
                assert!(
                    row.effects.starts_with("none"),
                    "{} must be effect-free (spec §4); got '{}'",
                    row.op,
                    row.effects
                );
            }
        }
    }

    /// The fallibility column of `summarize` and `is_green` is "total".
    /// (These are total functions over verdicts — spec §4.)
    #[test]
    fn aggregator_rows_are_total() {
        for row in MATRIX {
            if row.op == "summarize" || row.op == "is_green" {
                assert_eq!(
                    row.fallibility, "total",
                    "{} must be total (spec §4)",
                    row.op
                );
            }
        }
    }
}
