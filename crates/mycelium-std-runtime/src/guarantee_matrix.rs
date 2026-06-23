//! Per-operation guarantee matrix for `std.runtime` (ADR-020 v0 / SC-2 / VR-5).
//!
//! Every exported operation has an entry here. The matrix is asserted in tests, not
//! prose-only: any tag upgrade requires a checked theorem (VR-5) and a test update.

use mycelium_core::GuaranteeStrength;

/// One row in the guarantee matrix.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GaugeRow {
    pub operation: &'static str,
    pub strength: GuaranteeStrength,
    pub basis: &'static str,
}

/// Per-operation guarantee matrix for `std.runtime` v0.
///
/// Grounding: ADR-020 §4; RFC-0008 RT2 (sequentialization + Kahn-determinism differentials).
pub static MATRIX: &[GaugeRow] = &[
    GaugeRow {
        operation: "Scope::new",
        strength: GuaranteeStrength::Exact,
        basis: "constructor is trivially correct (no approximation)",
    },
    GaugeRow {
        operation: "Scope join semantics (all tasks complete before exit)",
        strength: GuaranteeStrength::Empirical,
        basis: "RT2 sequentialization differential; Kahn-determinism not yet Proven (ADR-020 §4)",
    },
    GaugeRow {
        operation: "Colony::new",
        strength: GuaranteeStrength::Exact,
        basis: "constructor is trivially correct",
    },
    GaugeRow {
        operation: "Colony Kahn-determinism (channel-mediated communication)",
        strength: GuaranteeStrength::Empirical,
        basis: "RT2 Kahn-determinism differential; formal proof pending (ADR-020 §4 FLAG)",
    },
    GaugeRow {
        operation: "Task purity contract",
        strength: GuaranteeStrength::Declared,
        basis: "asserted by caller; type system cannot enforce (VR-5: not upgraded without a checked basis)",
    },
    GaugeRow {
        operation: "TaskCtx::is_cancelled",
        strength: GuaranteeStrength::Exact,
        basis: "reads a boolean flag set by scope cancellation",
    },
    GaugeRow {
        operation: "Poll",
        strength: GuaranteeStrength::Exact,
        basis: "enum variant is the exact poll result",
    },
    GaugeRow {
        operation: "SweepOrder determinism",
        strength: GuaranteeStrength::Exact,
        basis: "sweep order is deterministic given the same queue state",
    },
    GaugeRow {
        operation: "Deadlock detection (DAG channels)",
        strength: GuaranteeStrength::Empirical,
        basis: "complete for DAG channel graphs; cyclic graphs FLAG (ADR-020 §7)",
    },
    GaugeRow {
        operation: "Sender::try_send / single-channel FIFO",
        strength: GuaranteeStrength::Exact,
        basis: "FIFO ordering within one channel is exact by construction",
    },
    GaugeRow {
        operation: "Receiver::try_recv / single-channel FIFO",
        strength: GuaranteeStrength::Exact,
        basis: "FIFO ordering within one channel is exact by construction",
    },
    GaugeRow {
        operation: "Network Kahn-determinism (cross-channel)",
        strength: GuaranteeStrength::Empirical,
        basis: "RT2 Kahn-determinism differential; formal proof pending (ADR-020 §4)",
    },
    // ── Channel construction ops (added with the real channel implementation) ──
    GaugeRow {
        operation: "Network::channel (construction)",
        strength: GuaranteeStrength::Exact,
        basis: "constructor is trivially correct; backed by Arc<Mutex<VecDeque>> (ADR-020 §4)",
    },
    GaugeRow {
        operation: "Network::channel zero-capacity check",
        strength: GuaranteeStrength::Exact,
        basis: "fail-closed: ZeroCapacity is returned deterministically when capacity==0 (G2/ADR-020 §4)",
    },
    GaugeRow {
        operation: "Sender::try_send FIFO (bounded channel)",
        strength: GuaranteeStrength::Exact,
        basis: "push to VecDeque tail; FIFO ordering exact by construction (ADR-020 §4)",
    },
    GaugeRow {
        operation: "Receiver::try_recv FIFO (bounded channel)",
        strength: GuaranteeStrength::Exact,
        basis: "pop from VecDeque head; FIFO ordering exact by construction (ADR-020 §4)",
    },
    // ── E12-1 execution maturity (M-709 scheduler / M-711 deadlock / M-713 supervision) ──
    GaugeRow {
        operation: "Scheduler RT2 sequentialization differential (OS threads)",
        strength: GuaranteeStrength::Empirical,
        basis: "parallel run equals sequential reference by RT1; property-tested, not Proven (M-709)",
    },
    GaugeRow {
        operation: "Scheduler backpressure bound (bounded ready queue)",
        strength: GuaranteeStrength::Exact,
        basis: "ready queue ≤ capacity by construction (enqueue only while len<capacity); G2 (M-709)",
    },
    GaugeRow {
        operation: "Scheduler liveness (each job runs exactly once)",
        strength: GuaranteeStrength::Empirical,
        basis: "property-tested over random job sets; not Proven (M-709)",
    },
    GaugeRow {
        operation: "Deadlock-freedom sweep (run_dataflow no-progress)",
        strength: GuaranteeStrength::Empirical,
        basis: "no-progress sweep ⇒ explicit Deadlock (never a hang, G2); complete for DAG graphs (M-711)",
    },
    GaugeRow {
        operation: "Supervision cancellation propagation (structured scope)",
        strength: GuaranteeStrength::Empirical,
        basis: "cooperative cancel cascades to every child; explicit outcome per child; property-tested (M-713)",
    },
    GaugeRow {
        operation: "Supervision restart bound (bounded cascade)",
        strength: GuaranteeStrength::Exact,
        basis: "rate + total restart bounds enforced structurally; inherited from M-356 Supervisor (M-713)",
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use mycelium_core::GuaranteeStrength;

    #[test]
    fn matrix_non_empty() {
        assert!(!MATRIX.is_empty(), "guarantee matrix must have entries");
    }

    #[test]
    fn task_purity_is_declared_not_higher() {
        let row = MATRIX
            .iter()
            .find(|r| r.operation == "Task purity contract")
            .expect("Task purity row must exist");
        assert_eq!(
            row.strength,
            GuaranteeStrength::Declared,
            "Task purity must not be upgraded beyond Declared without a checked basis (VR-5)"
        );
        // Mutant witness: changing strength to Empirical would make this test fail,
        // correctly catching an ungrounded tag upgrade.
    }

    #[test]
    fn kahn_determinism_is_empirical_not_proven() {
        for row in MATRIX {
            if row.operation.contains("Kahn") {
                assert_ne!(
                    row.strength,
                    GuaranteeStrength::Proven,
                    "Kahn-determinism must not be Proven without a checked theorem (VR-5): op={}",
                    row.operation
                );
                assert_ne!(
                    row.strength,
                    GuaranteeStrength::Exact,
                    "Kahn-determinism must not be Exact — it is Empirical (ADR-020 §4): op={}",
                    row.operation
                );
            }
        }
        // Mutant witness: setting any Kahn row to Proven would make this test fail.
    }

    #[test]
    fn no_reserved_vocabulary_in_operation_names() {
        // RFC-0008 §4.5 reserved vocabulary — must not appear in v0 public API.
        let reserved = [
            "hypha", "fuse", "xloc", "cyst", "graft", "forage", "backbone", "mesh", "tier",
            "reclaim",
        ];
        for row in MATRIX {
            for word in &reserved {
                assert!(
                    !row.operation.contains(word),
                    "Reserved vocabulary '{}' must not appear in v0 guarantee matrix (ADR-020 §5): op={}",
                    word,
                    row.operation
                );
            }
        }
    }

    #[test]
    fn test_new_channel_ops_are_exact() {
        // The four new bounded-channel operation rows are all Exact (deterministic by
        // construction). We match them by their known operation name prefixes, which are
        // distinct from the pre-existing Empirical rows (Kahn, Deadlock, Colony, etc.).
        // Mutant witness: changing any of these four rows to Empirical would make this test
        // fail, correctly catching an ungrounded tag downgrade for a deterministic operation.
        let exact_channel_op_prefixes = [
            "Network::channel (construction)",
            "Network::channel zero-capacity check",
            "Sender::try_send FIFO",
            "Receiver::try_recv FIFO",
        ];
        for prefix in &exact_channel_op_prefixes {
            let row = MATRIX
                .iter()
                .find(|r| r.operation.starts_with(prefix))
                .unwrap_or_else(|| panic!("guarantee matrix missing row starting with '{prefix}'"));
            assert_eq!(
                row.strength,
                GuaranteeStrength::Exact,
                "bounded-channel op '{}' must be Exact (ADR-020 §4)",
                row.operation
            );
        }
    }
}
