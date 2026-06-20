//! The **WIN / LOSS / REGRESSION classifier** — the honest core of the harness. For each non-baseline
//! backend on each case, it compares against the trusted interpreter and emits an explicit verdict.
//! This module is pure (no I/O, no timing of its own), so the classification is exhaustively unit-
//! testable and deterministic.
//!
//! The honesty discipline, made mechanical:
//! - A backend whose result **diverges** from the interpreter's is a **correctness LOSS** — recorded,
//!   never hidden. (A divergence is the worst kind of loss: a wrong answer faster is still wrong.)
//! - A backend that **cannot lower** the program is a **capability LOSS**, with the reason (G2).
//! - When both agree, the speed comparison yields **WIN** / **LOSS** / **NEUTRAL** vs the interpreter
//!   — tagged `Empirical`, with no pre-written target (VR-5). The "loss" here means *slower than the
//!   trusted in-process interpreter*, which for a trivial kernel is the expected, honest finding for
//!   a process-spawn-bound compiled path (M-602/E1) — surfaced with that reason, not buried.
//! - A backend that was only **skipped** (toolchain absent) yields no verdict — the harness could not
//!   measure it here; that is neither a win nor a loss.

use crate::backend::{observable_eq, Backend, Outcome};
use crate::timing::Timing;

/// The speed comparison band of a backend vs the interpreter, once both produced an *equal* value.
/// `Empirical`, with the threshold reified (no black box). A ratio is `interp_ns / backend_ns`:
/// `> 1` means the backend is faster than the interpreter (a win); `< 1` means slower (a loss).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Speed {
    /// Faster than the interpreter beyond the neutral band — a measured WIN.
    Win,
    /// Within the neutral band of the interpreter — neither a clear win nor loss.
    Neutral,
    /// Slower than the interpreter beyond the neutral band — a measured LOSS.
    Loss,
}

/// The full classification of one (backend, case) pair vs the trusted interpreter baseline.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum Verdict {
    /// Both produced equal values; the backend was faster than the interpreter.
    SpeedWin {
        /// `interp_ns / backend_ns` (`> 1`).
        ratio_x1000: u64,
    },
    /// Both produced equal values; within the neutral band.
    SpeedNeutral {
        /// `interp_ns / backend_ns`.
        ratio_x1000: u64,
    },
    /// Both produced equal values; the backend was slower than the interpreter (a speed LOSS).
    SpeedLoss {
        /// `interp_ns / backend_ns` (`< 1`).
        ratio_x1000: u64,
        /// A derivable reason, where one applies (e.g. process-spawn-bound) — honest, not buried.
        reason: String,
    },
    /// The backend produced a value that **diverges** from the interpreter's — a correctness LOSS.
    CorrectnessLoss {
        /// A short description of the divergence.
        detail: String,
    },
    /// The backend cannot lower this program — a capability LOSS, with the reason.
    CapabilityLoss {
        /// The backend's own explanation (the unlowerable-node reason).
        reason: String,
    },
    /// The backend errored at run time (overflow, depth limit, compile/exec failure).
    RuntimeError {
        /// The error message.
        message: String,
    },
    /// The backend was skipped (toolchain absent / feature off) — not measured, not a verdict.
    Skipped {
        /// Why it was skipped.
        reason: String,
    },
    /// The interpreter baseline itself failed on this case — no comparison is possible (the harness
    /// records it loudly; the trusted base should not fail, so this is a corpus/engine red flag).
    BaselineFailed {
        /// What the interpreter reported.
        message: String,
    },
}

impl Verdict {
    /// A short status word for the report table.
    #[must_use]
    pub fn status(&self) -> &'static str {
        match self {
            Verdict::SpeedWin { .. } => "WIN",
            Verdict::SpeedNeutral { .. } => "neutral",
            Verdict::SpeedLoss { .. } => "LOSS (speed)",
            Verdict::CorrectnessLoss { .. } => "LOSS (correctness)",
            Verdict::CapabilityLoss { .. } => "LOSS (capability)",
            Verdict::RuntimeError { .. } => "error",
            Verdict::Skipped { .. } => "skipped",
            Verdict::BaselineFailed { .. } => "baseline-failed",
        }
    }

    /// Whether this verdict counts as a LOSS (any of the three loss kinds) — for the "where we're
    /// losing" rollup. A skip / error / baseline-failure is not counted as a loss (it is its own
    /// category), and neutral/win are not losses.
    #[must_use]
    pub fn is_loss(&self) -> bool {
        matches!(
            self,
            Verdict::SpeedLoss { .. }
                | Verdict::CorrectnessLoss { .. }
                | Verdict::CapabilityLoss { .. }
        )
    }

    /// Whether this verdict counts as a WIN (a measured speed win).
    #[must_use]
    pub fn is_win(&self) -> bool {
        matches!(self, Verdict::SpeedWin { .. })
    }

    /// The honest guarantee tag for this verdict. A measured speed band is `Empirical`; a capability
    /// loss / runtime error / skip is `Declared` (an observed fact about the run, not a trial mean).
    #[must_use]
    pub fn guarantee_tag(&self) -> &'static str {
        match self {
            Verdict::SpeedWin { .. }
            | Verdict::SpeedNeutral { .. }
            | Verdict::SpeedLoss { .. }
            | Verdict::CorrectnessLoss { .. } => "Empirical",
            Verdict::CapabilityLoss { .. }
            | Verdict::RuntimeError { .. }
            | Verdict::Skipped { .. }
            | Verdict::BaselineFailed { .. } => "Declared",
        }
    }
}

/// The neutral band half-width: a backend within `[1/(1+NEUTRAL), 1+NEUTRAL]` of the interpreter's
/// time is `Neutral` (neither a clear win nor loss). 0.10 ⇒ within ±10%. Reified here (no black box);
/// a different study can pick a different band and say so.
pub const NEUTRAL_BAND: f64 = 0.10;

/// Encode a ratio as fixed-point parts-per-thousand (so the verdict is exactly serializable and
/// comparable without float noise in the report). `2.5x` ⇒ `2500`.
#[must_use]
fn ratio_x1000(interp_ns: f64, backend_ns: f64) -> u64 {
    if backend_ns <= 0.0 {
        return 0;
    }
    let r = interp_ns / backend_ns;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    {
        (r * 1000.0).round().max(0.0) as u64
    }
}

/// Classify one (backend, case) pair. `interp` is the trusted-baseline outcome+timing; `other` is the
/// backend under test. The baseline must be the interpreter (asserted in debug).
///
/// `backend` is the identity of `other` (used only to derive an honest speed-loss *reason*, e.g.
/// process-spawn-bound — never to change the verdict itself).
#[must_use]
pub fn classify(
    backend: Backend,
    interp: (&Outcome, Option<Timing>),
    other: (&Outcome, Option<Timing>),
) -> Verdict {
    debug_assert!(
        !backend.is_baseline(),
        "classify compares a NON-baseline backend"
    );
    let (interp_outcome, interp_timing) = interp;
    let (other_outcome, other_timing) = other;

    // 1. The trusted base must have produced a value to compare against.
    let interp_val = match interp_outcome {
        Outcome::Value(v) => v,
        Outcome::Skipped(m) | Outcome::Unlowerable(m) | Outcome::Error(m) => {
            return Verdict::BaselineFailed {
                message: format!("interpreter did not produce a value: {m}"),
            };
        }
    };

    // 2. The backend's outcome decides the verdict category (never-silent).
    let other_val = match other_outcome {
        Outcome::Value(v) => v,
        Outcome::Skipped(reason) => {
            return Verdict::Skipped {
                reason: reason.clone(),
            }
        }
        Outcome::Unlowerable(reason) => {
            return Verdict::CapabilityLoss {
                reason: reason.clone(),
            }
        }
        Outcome::Error(message) => {
            return Verdict::RuntimeError {
                message: message.clone(),
            }
        }
    };

    // 3. Differential correctness: a divergence from the trusted base is a correctness LOSS — the
    //    worst loss, recorded plainly (a wrong answer, however fast, is wrong). We compare on the
    //    OBSERVABLE (repr+payload+guarantee / content-identity), excluding dynamic Meta provenance —
    //    the same equivalence the M-210 checker + the three-way differential test use. (A full `==`
    //    would flag a spurious loss when a compiled backend stamps `Provenance::Root` on a read-back
    //    value vs the interpreter's `Derived` chain, though the result is identical.)
    if !observable_eq(interp_val, other_val) {
        return Verdict::CorrectnessLoss {
            detail: format!(
                "backend result diverges from the interpreter on the observable \
                 (interp={interp_val:?}, backend={other_val:?})"
            ),
        };
    }

    // 4. Both agree — compare speed. Without both timings we cannot band the speed (Neutral, honest).
    let (Some(it), Some(ot)) = (interp_timing, other_timing) else {
        return Verdict::SpeedNeutral { ratio_x1000: 1000 };
    };
    let r = it.ns_per_call / ot.ns_per_call;
    let x1000 = ratio_x1000(it.ns_per_call, ot.ns_per_call);

    if r > 1.0 + NEUTRAL_BAND {
        Verdict::SpeedWin { ratio_x1000: x1000 }
    } else if r < 1.0 / (1.0 + NEUTRAL_BAND) {
        // A speed loss — attach the honest derivable reason where one applies.
        let reason = if backend.is_process_spawn_bound() {
            "process-spawn-bound: the per-invocation time is dominated by spawning a fresh native \
             process, not kernel compute (M-602/E1) — expected for a trivial kernel vs the \
             in-process interpreter"
                .to_string()
        } else {
            "slower than the in-process interpreter on this case (measured; no target — VR-5)"
                .to_string()
        };
        Verdict::SpeedLoss {
            ratio_x1000: x1000,
            reason,
        }
    } else {
        Verdict::SpeedNeutral { ratio_x1000: x1000 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mycelium_core::{CoreValue, Meta, Payload, Provenance, Repr, Value};

    fn byte(b: u8) -> CoreValue {
        let bits: Vec<bool> = (0..8).map(|i| (b >> i) & 1 == 1).collect();
        CoreValue::Repr(
            Value::new(
                Repr::Binary { width: 8 },
                Payload::Bits(bits),
                Meta::exact(Provenance::Root),
            )
            .expect("valid byte"),
        )
    }

    fn timing(ns: f64) -> Timing {
        Timing {
            ns_per_call: ns,
            iters: 1000,
            batches: 5,
            ns_per_call_worst: ns,
        }
    }

    #[test]
    fn equal_values_and_faster_backend_is_a_speed_win() {
        let interp = Outcome::value_outcome(byte(0xAB));
        let other = Outcome::value_outcome(byte(0xAB));
        // interp 100ns, backend 25ns => 4x faster => WIN.
        let v = classify(
            Backend::Jit,
            (&interp, Some(timing(100.0))),
            (&other, Some(timing(25.0))),
        );
        assert!(v.is_win(), "expected a speed win, got {v:?}");
        assert_eq!(v.status(), "WIN");
        assert_eq!(v.guarantee_tag(), "Empirical");
        if let Verdict::SpeedWin { ratio_x1000 } = v {
            assert_eq!(ratio_x1000, 4000, "4.0x => 4000 per-mille");
        } else {
            panic!("not a SpeedWin");
        }
    }

    #[test]
    fn equal_values_and_slower_spawn_bound_backend_is_a_speed_loss_with_reason() {
        let interp = Outcome::value_outcome(byte(0xAB));
        let other = Outcome::value_outcome(byte(0xAB));
        // interp 100ns, direct-llvm 100000ns => far slower => LOSS, with the spawn-bound reason.
        let v = classify(
            Backend::DirectLlvm,
            (&interp, Some(timing(100.0))),
            (&other, Some(timing(100_000.0))),
        );
        assert!(v.is_loss(), "expected a loss, got {v:?}");
        match v {
            Verdict::SpeedLoss { reason, .. } => {
                assert!(
                    reason.contains("process-spawn-bound"),
                    "the spawn-bound reason must be surfaced honestly: {reason}"
                );
            }
            _ => panic!("expected SpeedLoss"),
        }
    }

    #[test]
    fn provenance_differences_are_not_correctness_losses() {
        use crate::backend::observable_eq;
        use mycelium_core::{ContentHash, Provenance};
        // The compiled backends read a value back and stamp `Provenance::Root`; the interpreter
        // records a `Derived` chain. Same repr+payload+guarantee ⇒ observationally equal ⇒ NOT a
        // correctness loss (the false positive this guards against).
        let payload = Payload::Bits((0..8).map(|i| (0xAB_u8 >> i) & 1 == 1).collect());
        let root = CoreValue::Repr(
            Value::new(
                Repr::Binary { width: 8 },
                payload.clone(),
                Meta::exact(Provenance::Root),
            )
            .unwrap(),
        );
        let derived = CoreValue::Repr(
            Value::new(
                Repr::Binary { width: 8 },
                payload,
                Meta::exact(Provenance::Derived {
                    op: ContentHash::parse("blake3:abc123").unwrap(),
                    inputs: vec![ContentHash::parse("blake3:def456").unwrap()],
                }),
            )
            .unwrap(),
        );
        assert!(
            observable_eq(&root, &derived),
            "values differing only in provenance must be observationally equal (not a loss)"
        );
        // And the classifier must treat them as equal — a speed verdict, never a correctness loss.
        let v = classify(
            Backend::DirectLlvm,
            (&Outcome::value_outcome(derived), Some(timing(100.0))),
            (&Outcome::value_outcome(root), Some(timing(100.0))),
        );
        assert!(
            !matches!(v, Verdict::CorrectnessLoss { .. }),
            "a provenance-only difference must NOT be a correctness loss, got {v:?}"
        );
    }

    #[test]
    fn diverging_values_is_a_correctness_loss_even_if_faster() {
        let interp = Outcome::value_outcome(byte(0xAB));
        let other = Outcome::value_outcome(byte(0xFF)); // wrong answer
                                                        // Backend is 10x faster, but the answer diverges — still a LOSS (correctness).
        let v = classify(
            Backend::Jit,
            (&interp, Some(timing(100.0))),
            (&other, Some(timing(10.0))),
        );
        assert!(v.is_loss());
        assert!(!v.is_win(), "a wrong-but-fast answer is never a win");
        assert!(matches!(v, Verdict::CorrectnessLoss { .. }));
        assert_eq!(v.status(), "LOSS (correctness)");
    }

    #[test]
    fn unlowerable_node_is_a_capability_loss() {
        let interp = Outcome::value_outcome(byte(0xAB));
        let other = Outcome::Unlowerable("unsupported node for the AOT subset: Fix".into());
        let v = classify(
            Backend::DirectLlvm,
            (&interp, Some(timing(100.0))),
            (&other, None),
        );
        assert!(v.is_loss());
        assert!(matches!(v, Verdict::CapabilityLoss { .. }));
        assert_eq!(v.guarantee_tag(), "Declared");
        if let Verdict::CapabilityLoss { reason } = v {
            assert!(
                reason.contains("Fix"),
                "the unlowerable reason must be kept: {reason}"
            );
        }
    }

    #[test]
    fn toolchain_absent_is_a_skip_not_a_loss() {
        let interp = Outcome::value_outcome(byte(0xAB));
        let other = Outcome::Skipped("native toolchain absent (clang)".into());
        let v = classify(
            Backend::DirectLlvm,
            (&interp, Some(timing(100.0))),
            (&other, None),
        );
        assert!(!v.is_loss(), "a skip is NOT a loss");
        assert!(!v.is_win());
        assert!(matches!(v, Verdict::Skipped { .. }));
        assert_eq!(v.status(), "skipped");
    }

    #[test]
    fn runtime_error_is_recorded_not_a_loss_category() {
        let interp = Outcome::value_outcome(byte(0xAB));
        let other = Outcome::Error("trit arithmetic overflowed fixed width".into());
        let v = classify(
            Backend::DirectLlvm,
            (&interp, Some(timing(100.0))),
            (&other, None),
        );
        assert!(matches!(v, Verdict::RuntimeError { .. }));
        assert!(!v.is_loss());
    }

    #[test]
    fn neutral_band_classifies_near_parity_as_neutral() {
        let interp = Outcome::value_outcome(byte(0xAB));
        let other = Outcome::value_outcome(byte(0xAB));
        // 100ns vs 105ns => within +-10% => Neutral.
        let v = classify(
            Backend::AotEnv,
            (&interp, Some(timing(100.0))),
            (&other, Some(timing(105.0))),
        );
        assert!(matches!(v, Verdict::SpeedNeutral { .. }), "got {v:?}");
        assert!(!v.is_loss() && !v.is_win());
    }

    #[test]
    fn baseline_failure_is_flagged_loudly() {
        let interp = Outcome::Error("interpreter blew up".into());
        let other = Outcome::value_outcome(byte(0xAB));
        let v = classify(
            Backend::AotEnv,
            (&interp, None),
            (&other, Some(timing(10.0))),
        );
        assert!(matches!(v, Verdict::BaselineFailed { .. }));
    }

    #[test]
    fn missing_timings_with_equal_values_is_neutral_not_a_false_win() {
        // If we couldn't time one side (e.g. a one-shot run), equal values => Neutral, never a
        // fabricated speed verdict.
        let interp = Outcome::value_outcome(byte(0xAB));
        let other = Outcome::value_outcome(byte(0xAB));
        let v = classify(Backend::AotEnv, (&interp, None), (&other, None));
        assert!(matches!(v, Verdict::SpeedNeutral { ratio_x1000: 1000 }));
    }
}
