//! The `std.spore` guarantee matrix — RFC-0016 §4.5 / spec §4.
//!
//! Encodes the per-op guarantee tags from the **normative spec §4 table** as **checked data**,
//! not prose only (C2 / VR-5). Tests assert coverage and tag correctness, mirroring the table
//! in `docs/spec/stdlib/spore.md §4.5`.
//!
//! # Tag legend (VR-5 / RFC-0016 C2)
//!
//! - **`Exact`** — deterministic, no accuracy semantics. A content hash is a pure function of
//!   normalized structure (RFC-0001 §4.6). Build / identity / manifest-authoring rows are `Exact`.
//! - **`Empirical` (≤ ceiling)** — the reconstruction row inherits the tag `std.vsa` establishes
//!   for the probabilistic resonator decode (FR-C2). `spore` carries, never sets, this tag.
//! - **`Exact` (via verification)** — the deploy row verifies the deterministic hash at the target;
//!   the row's `Exact` is the *verification*'s tag. **FLAGGED §7 Q2** — M-620 owned.
//!
//! # Fallibility (C1 / G2)
//!
//! Every `PublishErr`-fallible row names the specific offending input (surfaceless phylum, hashless
//! dep, version/hash disagreement, bad include, cycle, no sources). No row permits a silent drop
//! of an error. The `deploy` row is `FLAGGED §7 Q2` (Phase-6, M-620).
//!
//! # EXPLAIN-able? (C3 / G11)
//!
//! An EXPLAIN artifact exists for every selecting/converting/approximating op. The `identity` hash
//! itself is the receipt. The `manifest_of` record is the inspectable artifact. A publish/deploy
//! refusal carries a diagnostic naming the offending input.

/// Guarantee tag string — the lattice position (`Exact ⊐ Proven ⊐ Empirical ⊐ Declared`).
/// The reconstruction row additionally notes the `std.vsa` ceiling (FR-C2).
pub type GuaranteeTag = &'static str;

/// One row of the `std.spore` guarantee matrix (RFC-0016 §4.5 / spec §4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatrixRow {
    /// The exported operation name (matching spec §3 / §4 surface labels).
    pub op: &'static str,
    /// Guarantee tag on `Exact ⊐ Proven ⊐ Empirical ⊐ Declared` (VR-5).
    pub guarantee: GuaranteeTag,
    /// The explicit error/fallibility description (C1 — named, never silent).
    pub fallibility: &'static str,
    /// Declared effects (C6). "none" = no effects; "io" = reads/writes files or network.
    pub effects: &'static str,
    /// Whether the op has a C3 EXPLAIN artifact (G11).
    pub explain_able: bool,
    /// The never-silent property: how errors/none-cases are handled. No "silently dropped" allowed.
    pub never_silent_property: &'static str,
}

/// The `std.spore` guarantee matrix (spec §4.5), encoded as data (RFC-0016 §4.5).
///
/// One row per exported op (grouped: build / manifest / regrowth / deploy). Tag = the spec §4
/// normative entry. `explain_able` = whether the op exposes an inspectable artifact (C3).
/// Tests assert coverage and tag correctness so divergence from the spec is caught mechanically
/// (VR-5 / C2).
///
/// # Deploy row (FLAG Q2)
///
/// The `deploy` row is FLAGGED — the full native-deploy half (turning a spore into a running colony)
/// is Phase-6-gated (M-620). The row is present to fix the seam and carry the honesty commitment
/// (hash mismatch = explicit `DeployErr::HashMismatch`, never silent); it is not implemented here.
///
/// # Regrowth row (FLAG Q4)
///
/// The `reconstruct (via std.vsa)` row shows the bound `std.vsa` establishes. `spore` packages the
/// manifest; `vsa` performs the decode. The `Empirical` ceiling is owned by FR-C2 / M-513.
pub const MATRIX: &[MatrixRow] = &[
    // ─── Build / project packaging ─────────────────────────────────────────────────────────────
    MatrixRow {
        op: "build (project → spore)",
        guarantee: "Exact (deterministic hash; metadata-invariant — ADR-003)",
        fallibility: "Err(PublishErr::*) — surfaceless phylum, hashless dep, version/hash \
                      disagreement, bad include, dep cycle, no sources",
        effects: "io (reads project; writes artifact)",
        explain_able: true,
        never_silent_property: "every missing/ambiguous input is a typed PublishErr naming the \
                                 offending input; no partial artifact is ever written (M-368 §5)",
    },
    MatrixRow {
        op: "build_value (spore(v) — degenerate ADR-013 §2)",
        guarantee: "Exact (deterministic hash)",
        fallibility: "Err(PublishErr::*) (value-payload subset; currently infallible for \
                      well-formed values — reserved for future validation)",
        effects: "none (pure construction; no IO)",
        explain_able: true,
        never_silent_property: "no partial spore is ever written; errors are explicit (C1/G2)",
    },
    // ─── Identity / inspection ──────────────────────────────────────────────────────────────────
    MatrixRow {
        op: "identity (spore hash)",
        guarantee: "Exact (deterministic)",
        fallibility: "total",
        effects: "none",
        explain_able: true,
        never_silent_property: "total — no failure mode; the hash itself is the identity receipt",
    },
    MatrixRow {
        op: "explain",
        guarantee: "Exact (deterministic; a total function of manifest + DAG)",
        fallibility: "total",
        effects: "none",
        explain_able: true,
        never_silent_property: "the EXPLAIN itself is the artifact — deterministic, no hidden state",
    },
    // ─── Manifest read / verify / inspect ──────────────────────────────────────────────────────
    MatrixRow {
        op: "manifest_of",
        guarantee: "Exact (deterministic)",
        fallibility: "None when the spore carries no manifest — never a fabricated empty one (C1)",
        effects: "none",
        explain_able: true,
        never_silent_property: "None is an honest absence; never a fabricated empty manifest (C1/G2)",
    },
    MatrixRow {
        op: "validate (manifest)",
        guarantee: "Exact (deterministic schema + invariant check)",
        fallibility: "Err(MalformedManifest::*) — bad mode/bound, missing recipe/decode param, \
                      ResonatorOverStrength (FR-C2 ceiling violated)",
        effects: "none",
        explain_able: true,
        never_silent_property: "named error per invariant (C1/G11); ResonatorOverStrength names \
                                 the FR-C2 violation explicitly — never silently downgraded",
    },
    MatrixRow {
        op: "manifest_hash",
        guarantee: "Exact (deterministic)",
        fallibility: "total",
        effects: "none",
        explain_able: false,
        never_silent_property: "total — no failure mode",
    },
    MatrixRow {
        op: "mode (read)",
        guarantee: "Exact (deterministic)",
        fallibility: "total",
        effects: "none",
        explain_able: false,
        never_silent_property: "total — enum accessor, no failure mode",
    },
    MatrixRow {
        op: "declared_strength (read)",
        guarantee: "Exact (deterministic)",
        fallibility: "total",
        effects: "none",
        explain_able: false,
        never_silent_property: "total — enum accessor; the ceiling (≤ Empirical for Resonator) is \
                                 enforced at construction, not at read time",
    },
    // ─── Regrowth (the seam — std.vsa performs the decode; spore carries the manifest) ─────────
    MatrixRow {
        op: "reconstruct (via std.vsa — spec §4.5 seam row; FLAG Q4)",
        guarantee: "Empirical (probabilistic resonator decode) / Exact (indexed-retrieval exact-hit) \
                    — owned by std.vsa (M-513), bounded by FR-C2; spore carries, never sets the tag",
        fallibility: "Err/refusal on non-convergence (a std.vsa op; spore does not perform it)",
        effects: "(vsa's effects)",
        explain_able: true,
        never_silent_property: "the carried {ε,δ,strength} bound + decode params are the inspectable \
                                 artifact (C3); non-convergence is an explicit VsaError (never silent)",
    },
    // ─── Deploy (Phase-6, M-620 — FLAGGED §7 Q2) ───────────────────────────────────────────────
    MatrixRow {
        op: "deploy (Phase-6 native path — FLAG Q2: M-620; NOT implemented here)",
        guarantee: "Exact (deploy verifies the deterministic hash) — FLAGGED §7 Q2",
        fallibility: "Err(DeployErr::HashMismatch{expected, got}), TargetUnavailable, Unsupported",
        effects: "io (network/native deploy)",
        explain_able: true,
        never_silent_property: "hash mismatch on deploy is an explicit DeployErr naming both \
                                 hashes — no silent overwrite, no partial deploy (C1/G2); \
                                 FLAGGED §7 Q2 — not yet implemented (M-620 Phase-6)",
    },
];

#[cfg(test)]
mod tests {
    use super::MATRIX;

    /// The matrix covers all 11 spec §4.5 op rows (the normative table).
    /// Guard: adding or removing a row without updating this count makes it fail.
    #[test]
    fn matrix_covers_all_spec_ops() {
        // The spec §4.5 table has 10 normative rows (build, build_value, identity, explain,
        // manifest_of, validate, manifest_hash, mode, declared_strength, reconstruct, deploy).
        let expected_op_prefixes = [
            "build (project",
            "build_value",
            "identity",
            "explain",
            "manifest_of",
            "validate",
            "manifest_hash",
            "mode",
            "declared_strength",
            "reconstruct",
            "deploy",
        ];
        for prefix in &expected_op_prefixes {
            assert!(
                MATRIX.iter().any(|r| r.op.starts_with(prefix)),
                "matrix is missing op starting with {prefix:?} (spec §4.5)"
            );
        }
        assert_eq!(
            MATRIX.len(),
            11,
            "expected exactly 11 rows (spec §4.5 table); got {}",
            MATRIX.len()
        );
    }

    /// All `Exact` rows start with "Exact" in their guarantee tag (VR-5).
    /// Guard: a tag that doesn't start with "Exact" for a deterministic op makes this fail.
    #[test]
    fn exact_rows_start_with_exact() {
        // The build/identity/manifest rows are all Exact; only the reconstruct and deploy rows
        // are flagged/Empirical. We enumerate the expected Exact rows.
        let exact_op_prefixes = [
            "build (project",
            "build_value",
            "identity",
            "explain",
            "manifest_of",
            "validate",
            "manifest_hash",
            "mode",
            "declared_strength",
            "deploy", // Exact via verification (FLAG Q2)
        ];
        for prefix in &exact_op_prefixes {
            let row = MATRIX
                .iter()
                .find(|r| r.op.starts_with(prefix))
                .unwrap_or_else(|| panic!("row {prefix:?} missing"));
            assert!(
                row.guarantee.starts_with("Exact"),
                "op {:?} must start its guarantee tag with 'Exact'; got {:?}",
                row.op,
                row.guarantee
            );
        }
    }

    /// The `reconstruct` row's guarantee mentions `Empirical` (FR-C2 ceiling).
    /// Guard: a `reconstruct` row without `Empirical` in its tag violates FR-C2 transparency.
    #[test]
    fn reconstruct_row_mentions_empirical_ceiling() {
        // Mutant witness: removing "Empirical" from the reconstruct row tag makes this fail.
        let row = MATRIX
            .iter()
            .find(|r| r.op.starts_with("reconstruct"))
            .expect("reconstruct row must be in the matrix (spec §4.5)");
        assert!(
            row.guarantee.contains("Empirical"),
            "reconstruct row must mention 'Empirical' (FR-C2 ceiling): {:?}",
            row.guarantee
        );
    }

    /// The `validate` row's fallibility mentions `ResonatorOverStrength` (FR-C2).
    /// Guard: not naming ResonatorOverStrength makes the error set incomplete (C1/G11).
    #[test]
    fn validate_row_names_resonator_over_strength() {
        // Mutant witness: removing ResonatorOverStrength from validate's fallibility hides the
        // FR-C2 enforcement contract.
        let row = MATRIX
            .iter()
            .find(|r| r.op.starts_with("validate"))
            .expect("validate row must be in the matrix");
        assert!(
            row.fallibility.contains("ResonatorOverStrength"),
            "validate fallibility must name ResonatorOverStrength (FR-C2): {:?}",
            row.fallibility
        );
    }

    /// The `deploy` row is flagged (§7 Q2 — M-620 not yet implemented).
    /// Guard: an unflagged deploy row would overclaim scope.
    #[test]
    fn deploy_row_is_flagged() {
        // Mutant witness: removing FLAG/FLAGGED from the deploy row or guarantee makes this fail.
        let row = MATRIX
            .iter()
            .find(|r| r.op.starts_with("deploy"))
            .expect("deploy row must be in the matrix (spec §4.5)");
        assert!(
            row.op.contains("FLAG") || row.guarantee.contains("FLAG"),
            "deploy row must be flagged (§7 Q2 / M-620): op={:?}, guarantee={:?}",
            row.op,
            row.guarantee
        );
    }

    /// Every row has a non-empty `never_silent_property` (C1/G2).
    /// Guard: leaving never_silent_property empty on any row makes this fail.
    #[test]
    fn every_row_has_nonempty_never_silent_property() {
        for row in MATRIX {
            assert!(
                !row.never_silent_property.is_empty(),
                "row {:?} must state its never_silent_property (C1/G2)",
                row.op
            );
        }
    }

    /// Every row's `op` and `guarantee` fields are non-empty (basic completeness).
    #[test]
    fn every_row_has_nonempty_op_and_guarantee() {
        for row in MATRIX {
            assert!(
                !row.op.is_empty(),
                "every MatrixRow must have a non-empty op"
            );
            assert!(
                !row.guarantee.is_empty(),
                "every MatrixRow must have a non-empty guarantee (op={:?})",
                row.op
            );
        }
    }

    /// The manifest-carrying rows have `explain_able: true` (C3 — they expose an inspectable
    /// artifact).
    #[test]
    fn manifest_rows_are_explain_able() {
        let explain_ops = [
            "build (project",
            "build_value",
            "identity",
            "explain",
            "manifest_of",
            "validate",
            "reconstruct",
            "deploy",
        ];
        for prefix in &explain_ops {
            let row = MATRIX
                .iter()
                .find(|r| r.op.starts_with(prefix))
                .unwrap_or_else(|| panic!("row {prefix:?} missing"));
            assert!(
                row.explain_able,
                "op {:?} must be explain_able (C3/G11): {:?}",
                row.op, row
            );
        }
    }

    /// Accessor rows (`manifest_hash`, `mode`, `declared_strength`) have `explain_able: false`
    /// — they are pure reads with no selection/approximation needing EXPLAIN.
    #[test]
    fn accessor_rows_are_not_explain_able() {
        let not_explain_ops = ["manifest_hash", "mode", "declared_strength"];
        for prefix in &not_explain_ops {
            let row = MATRIX
                .iter()
                .find(|r| r.op.starts_with(prefix))
                .unwrap_or_else(|| panic!("row {prefix:?} missing"));
            assert!(
                !row.explain_able,
                "accessor op {:?} should not be explain_able (no selection/approximation): {:?}",
                row.op, row
            );
        }
    }

    /// The matrix `MatrixRow` type is `Debug + Clone + PartialEq + Eq` (derive sanity).
    #[test]
    fn matrix_row_derives_are_correct() {
        let r1 = MATRIX[0].clone();
        let r2 = MATRIX[0].clone();
        assert_eq!(r1, r2, "cloned rows must be equal");
        let _ = format!("{r1:?}"); // Debug
    }
}
