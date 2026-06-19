//! Verdict types for `std.testing` (M-534, #174).
//!
//! The `Verdict` sum is the load-bearing type: **`Skipped` and `Undetermined` are first-class
//! variants**, not flavours of `Pass` (spec §3 / §4.1 C1 — the honesty crux). Every non-`Pass`
//! verdict is a reified, inspectable artifact (C3/G11/SC-3).

// ─── FailRecord ────────────────────────────────────────────────────────────────

/// A structured failure record carried by [`Verdict::Fail`].
///
/// This is the `std.testing` representation of a diagnostic record (spec §3:
/// `Fail { diagnostic: Diag }`).
///
/// # FLAG-DIAG (RESOLVED, testing↔diag seam — spec §7-Q2)
/// `std.diag` (M-510) has landed, so `FailRecord` now **delegates** to its canonical record:
/// [`FailRecord::to_diag`] projects to a [`mycelium_diag::Diag`] — the structured diagnostic the
/// rest of the failure-legibility substrate speaks (README §5). `FailRecord` keeps the
/// **testing-specific reproduction metadata** (the seed + trial index) that a generic `Diag` does
/// not model, and folds them — with the description and op context — into the `Diag`'s message and
/// notes. The `Diag` is the legibility artifact; this record is its seed-reproducible test wrapper.
///
/// # C1 — never-silent
/// Every failure is a structured record with a description + reproducing seed — never an opaque
/// red/green bit (RFC-0013 via spec §4.1 C3).
///
/// # C3 — EXPLAIN
/// The `description` carries the shrunk counterexample or diff; the `seed` makes the failure
/// reproducible; the `context` names the op that failed. This is the EXPLAIN artifact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FailRecord {
    /// Human-readable description of the failure (shrunk counterexample, diff, etc.).
    /// **EXPLAIN artifact** (C3/G11/SC-3).
    pub description: String,
    /// The seed that reproduces this failure (RT3 / spec §3 — "reproducible by seed").
    pub seed: u64,
    /// The trial index at which the failure occurred.
    pub trial: u32,
    /// The op context that produced this failure (e.g. `"for_all"`, `"golden(name)"`).
    pub context: String,
}

impl FailRecord {
    /// Project this failure to the canonical [`mycelium_diag::Diag`] record (the testing↔diag
    /// seam — spec §7-Q2). The `description` becomes the diagnostic message; the op `context`, the
    /// reproducing `seed`, and the `trial` index ride along as EXPLAIN notes (G11). Severity is
    /// `Error` and the code is the test-failure class — never an opaque red/green bit (C1/C3).
    ///
    /// # Guarantee tag: `Exact` (a pure, total projection)
    #[must_use]
    pub fn to_diag(&self) -> mycelium_diag::Diag {
        mycelium_diag::Diag::error(mycelium_diag::Code::Other("test.fail".to_owned()))
            .message(self.description.clone())
            .note(format!("context={}", self.context))
            .note(format!("seed={}", self.seed))
            .note(format!("trial={}", self.trial))
    }
}

// ─── SkipReason ───────────────────────────────────────────────────────────────

/// The reason a test was skipped (spec §3).
///
/// A `Skipped` verdict always carries a reason — **never** an absent or unnamed skip (C1/G2).
/// The reason is part of the EXPLAIN artifact; a skip without a reason would be a black box
/// (C3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkipReason {
    /// The test is explicitly marked as ignored (e.g. `#[ignore]` equivalent).
    Ignored,
    /// A precondition for this test was not met (e.g. a required feature is absent).
    UnmetPrecondition,
    /// The golden baseline does not exist and must be recorded before the test can run.
    /// **A missing baseline is `NeedsRecord`, never a silent auto-accept (C1/G2 — the golden
    /// test honesty crux).**
    NeedsRecord,
    /// A required backend for a differential test is not available in this environment.
    /// **An unavailable backend is `BackendUnavailable`, never a silent pass (C1/G2 — the
    /// differential test honesty crux).**
    BackendUnavailable,
    /// A tool required to run this test is missing from the environment.
    ToolMissing,
}

// ─── UndetReason ──────────────────────────────────────────────────────────────

/// The reason a test result is undetermined (ran but could not decide — spec §3).
///
/// `Undetermined` is distinct from both `Pass` and `Skipped`:
/// - `Skipped` = could not run.
/// - `Undetermined` = ran but could not reach a verdict.
///
/// A non-decision is **never** a `Pass` (C1/G2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UndetReason {
    /// The oracle required for a differential check was unavailable at runtime.
    OracleUnavailable,
    /// The property budget was exhausted without finding a counterexample, but the search was
    /// inconclusive (e.g. the generator space was undersampled relative to the property's domain).
    BudgetExhaustedInconclusive,
    /// The input to a differential or property is non-deterministic and could not be replayed.
    /// **A flaky non-deterministic input is `NonDeterministicInput`, never a silent pass (RT3 /
    /// C6 — the seeded-generator reproducibility discipline).**
    NonDeterministicInput,
}

// ─── Verdict ──────────────────────────────────────────────────────────────────

/// The outcome of a single test case (spec §3 / §4 guarantee matrix).
///
/// **The honesty crux:** `Skipped` and `Undetermined` are **first-class variants**, not flavours
/// of `Pass`. The aggregator ([`crate::summarize`]) keeps their counts distinct; [`crate::is_green`]
/// surfaces them. "Green" therefore means *checked and passed*, never *did not check* (C1/G2).
///
/// # C3 — EXPLAIN
/// Every non-`Pass` variant is a reified, inspectable artifact:
/// - `Fail` — carries a [`FailRecord`] with description, seed, trial, context.
/// - `Skipped` — carries a [`SkipReason`] (never a nameless skip).
/// - `Undetermined` — carries an [`UndetReason`] (never a nameless non-decision).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verdict {
    /// The check ran and all assertions passed.
    Pass,
    /// The check ran and at least one assertion failed.
    ///
    /// Carries a [`FailRecord`] with the structured diagnostic: description, reproducing seed,
    /// trial index, and op context. **FLAG-DIAG:** this will delegate to `std.diag` (M-510) when
    /// that crate lands.
    Fail {
        /// The structured failure record (spec §3 `Fail { diagnostic: Diag }`).
        record: FailRecord,
    },
    /// The check could not run. **Reported, never silently absent (C1/G2 — the honesty crux).**
    Skipped {
        /// Why the check was skipped (never anonymous — C3/G11).
        reason: SkipReason,
    },
    /// The check ran but could not reach a verdict (e.g. oracle unavailable, budget exhausted
    /// inconclusively). **Not a `Pass` (C1/G2).**
    Undetermined {
        /// Why the verdict could not be determined.
        reason: UndetReason,
    },
}

// ─── Summary ──────────────────────────────────────────────────────────────────

/// The aggregated outcome of a collection of verdicts (spec §3 / [`crate::summarize`]).
///
/// # The crux
/// `skipped` and `undetermined` are **distinct from `passed`** — a `Summary` cannot be used
/// to claim "all passed" when tests were skipped (C1/G2). See [`crate::is_green`].
///
/// # EXPLAIN
/// The per-class counts are the EXPLAIN artifact: a caller can inspect them to see how many
/// tests ran, how many were skipped, and why the suite is or is not green.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Summary {
    /// Number of verdicts that were [`Verdict::Pass`].
    pub passed: u32,
    /// Number of verdicts that were [`Verdict::Fail`].
    pub failed: u32,
    /// Number of verdicts that were [`Verdict::Skipped`] (**distinct from passed**).
    pub skipped: u32,
    /// Number of verdicts that were [`Verdict::Undetermined`] (**distinct from passed**).
    pub undetermined: u32,
}

impl Summary {
    /// Total number of verdicts in this summary.
    #[must_use]
    pub fn total(&self) -> u32 {
        self.passed
            .saturating_add(self.failed)
            .saturating_add(self.skipped)
            .saturating_add(self.undetermined)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verify `SkipReason` variants are distinct (no accidental merging).
    #[test]
    fn skip_reasons_are_distinct() {
        use SkipReason::*;
        let reasons = [
            Ignored,
            UnmetPrecondition,
            NeedsRecord,
            BackendUnavailable,
            ToolMissing,
        ];
        for i in 0..reasons.len() {
            for j in 0..reasons.len() {
                if i != j {
                    assert_ne!(
                        reasons[i], reasons[j],
                        "SkipReason variants must be distinct"
                    );
                }
            }
        }
    }

    /// Verify `UndetReason` variants are distinct.
    #[test]
    fn undet_reasons_are_distinct() {
        use UndetReason::*;
        let reasons = [
            OracleUnavailable,
            BudgetExhaustedInconclusive,
            NonDeterministicInput,
        ];
        for i in 0..reasons.len() {
            for j in 0..reasons.len() {
                if i != j {
                    assert_ne!(
                        reasons[i], reasons[j],
                        "UndetReason variants must be distinct"
                    );
                }
            }
        }
    }

    /// `Summary::total()` equals the sum of all counts.
    #[test]
    fn summary_total_equals_sum() {
        let s = Summary {
            passed: 3,
            failed: 1,
            skipped: 2,
            undetermined: 1,
        };
        assert_eq!(s.total(), 7);
    }

    /// Default `Summary` has all-zero counts.
    #[test]
    fn summary_default_is_zero() {
        let s = Summary::default();
        assert_eq!(s.total(), 0);
    }

    /// `Verdict::Pass` is not equal to `Verdict::Skipped` (the honesty crux).
    #[test]
    fn verdict_pass_ne_skipped_all_reasons() {
        for reason in [
            SkipReason::Ignored,
            SkipReason::UnmetPrecondition,
            SkipReason::NeedsRecord,
            SkipReason::BackendUnavailable,
            SkipReason::ToolMissing,
        ] {
            assert_ne!(
                Verdict::Pass,
                Verdict::Skipped { reason },
                "Verdict::Pass must never equal Skipped{{reason={reason:?}}} (honesty crux)"
            );
        }
    }

    /// `Verdict::Pass` is not equal to `Verdict::Undetermined` (the honesty crux).
    #[test]
    fn verdict_pass_ne_undetermined_all_reasons() {
        for reason in [
            UndetReason::OracleUnavailable,
            UndetReason::BudgetExhaustedInconclusive,
            UndetReason::NonDeterministicInput,
        ] {
            assert_ne!(
                Verdict::Pass,
                Verdict::Undetermined { reason },
                "Verdict::Pass must never equal Undetermined{{reason={reason:?}}} (honesty crux)"
            );
        }
    }

    /// `Verdict::Fail` carries its record fields accurately.
    #[test]
    fn verdict_fail_record_fields() {
        let record = FailRecord {
            description: "test failure".to_owned(),
            seed: 42,
            trial: 3,
            context: "for_all".to_owned(),
        };
        let v = Verdict::Fail {
            record: record.clone(),
        };
        if let Verdict::Fail { record: r } = v {
            assert_eq!(r.description, "test failure");
            assert_eq!(r.seed, 42);
            assert_eq!(r.trial, 3);
            assert_eq!(r.context, "for_all");
        } else {
            panic!("expected Fail");
        }
    }

    /// `FailRecord::to_diag` delegates to the canonical `mycelium_diag::Diag` (testing↔diag seam,
    /// spec §7-Q2): the description is the message; context/seed/trial ride along as notes; the
    /// severity is `Error` — never an opaque red/green bit (C1/C3).
    #[test]
    fn fail_record_projects_to_diag() {
        let record = FailRecord {
            description: "shrunk counterexample: n=7".to_owned(),
            seed: 42,
            trial: 3,
            context: "for_all".to_owned(),
        };
        let d = record.to_diag();
        assert_eq!(d.severity(), mycelium_diag::Severity::Error);
        assert_eq!(d.message, "shrunk counterexample: n=7");
        // The reproduction metadata survives in the diagnostic's EXPLAIN notes.
        assert!(d.notes.iter().any(|n| n == "seed=42"));
        assert!(d.notes.iter().any(|n| n == "trial=3"));
        assert!(d.notes.iter().any(|n| n == "context=for_all"));
    }
}
