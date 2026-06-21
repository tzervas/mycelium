//! `adk` guarantee matrix — one [`MatrixRow`] per exported op (RFC-0023 §6.1).
//!
//! The matrix is the **load-bearing C2/VR-5 deliverable** for the `adk` phylum:
//! guarantee tags are asserted in tests, not prose-only.  The format follows
//! `crates/mycelium-std-io/src/guarantee_matrix.rs` exactly.
//!
//! # Tag justification summary (VR-5 — downgrade rather than overclaim)
//!
//! | Tag | Rows | Reason |
//! |---|---|---|
//! | `Exact` | `put`, `append_event`, `is_synthetic`, `model_allowed_tags`, `run` (type)  | Pure, total, no accuracy semantics (RFC-0016 C2). |
//! | `Declared` | `LlmOutcome` (content tag), `generate` (stub) | LLM output is asserted without a checked basis (VR-5 / RFC-0023 §6.1); `generate` is deferred (M-381/M-646). |
//! | `Empirical` | `LlmOutcome` (tag when trial-validated) | Round-trip quality established by trial, not theorem (VR-5). |
//!
//! # THE LOAD-BEARING GUARD (RFC-0023 §6.1)
//! Every LLM-output row's tag MUST be `Declared` or `Empirical` — NEVER `Proven` or `Exact`.
//! The test [`llm_output_tag_is_declared_or_empirical`] fails if any LLM-output row
//! is tagged `Proven` or `Exact`.  This is the mechanical enforcement of RFC-0023 §6.1 in code.
//!
//! # Effect column
//! - `"none"` — pure; no OS facility touched.
//! - `"io"` — touches an external substrate (deferred; graft not yet active).
//!
//! # EXPLAIN column
//! - `"yes"` — the op carries a diagnostic record or a refusal reason.
//! - `"n/a"` — pure faithful operation; no selection/conversion/approximation.
//!
//! ## Design spec
//! `docs/rfcs/RFC-0023-Agent-Development-Kit-Phylum.md` §6.1

// ── Tag types (matching std.io format) ───────────────────────────────────────

/// Guarantee tag on the honesty lattice `Exact ⊐ Proven ⊐ Empirical ⊐ Declared` (VR-5).
///
/// This is a self-contained copy of the lattice for use in the guarantee matrix —
/// it does not depend on `mycelium-core`'s `GuaranteeStrength` to keep the matrix
/// independently verifiable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuaranteeTag {
    /// No accuracy / precision / probability semantics (RFC-0016 C2 — "no accuracy semantics → Exact").
    Exact,
    /// The property is established by a **checked theorem**.  Not used here — no theorem
    /// with checked side-conditions exists for any `adk` op.
    Proven,
    /// The property holds over a **generated corpus** (proptest); not `Proven` because
    /// no theorem with checked side-conditions exists (VR-5).
    Empirical,
    /// The property is **asserted without a checked basis**; always FLAGGED (VR-5).
    Declared,
}

impl GuaranteeTag {
    /// Human-readable name matching the lattice notation.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            GuaranteeTag::Exact => "Exact",
            GuaranteeTag::Proven => "Proven",
            GuaranteeTag::Empirical => "Empirical",
            GuaranteeTag::Declared => "Declared",
        }
    }

    /// `true` iff this tag is allowed for an LLM output row (RFC-0023 §6.1).
    #[must_use]
    pub fn is_allowed_for_llm_output(self) -> bool {
        matches!(self, GuaranteeTag::Declared | GuaranteeTag::Empirical)
    }
}

/// Fallibility classification (C1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fallibility {
    /// The op cannot fail for any well-formed input.
    Total,
    /// The op returns an explicit `Result` or `Option`; the error set is named.
    Fallible,
}

/// Whether the op surfaces an EXPLAIN artifact (C3).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Explainable {
    /// The op carries a diagnostic record / refusal reason (machine-legible EXPLAIN surface).
    Yes,
    /// The op neither selects, converts, nor approximates; no hidden decision (C3 n/a).
    NotApplicable,
}

/// Whether a matrix row covers an LLM-output value.
///
/// LLM-output rows are subject to the extra constraint: tag ∈ {Declared, Empirical}.
/// This field is the load-bearing flag for the guard test.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IsLlmOutput {
    /// This row covers an LLM output value (tag MUST be Declared or Empirical).
    Yes,
    /// This row does not cover an LLM output value (tag may be any honest value).
    No,
}

/// One row in the `adk` guarantee matrix (RFC-0023 §6.1 / RFC-0016 §4.5).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatrixRow {
    /// Exported operation name (nodule.op format).
    pub op: &'static str,
    /// Guarantee tag (VR-5).
    pub guarantee: GuaranteeTag,
    /// Fallibility: total or fallible.
    pub fallibility: Fallibility,
    /// The explicit error set (empty string for total ops).
    pub error_set: &'static str,
    /// Declared effects: `"none"` or `"io"`.
    pub effects: &'static str,
    /// Whether the op surfaces a C3 EXPLAIN artifact.
    pub explainable: Explainable,
    /// Whether this row covers an LLM-output value (the honesty guard — RFC-0023 §6.1).
    pub llm_output: IsLlmOutput,
}

/// The `adk` guarantee matrix.
///
/// One row per exported op.  Asserted in tests — never prose-only (C2 / VR-5).
///
/// ## LLM-output rows (the load-bearing guard)
/// Rows with `llm_output: IsLlmOutput::Yes` MUST have `guarantee ∈ {Declared, Empirical}`.
/// The test `llm_output_tag_is_declared_or_empirical` enforces this.
pub const MATRIX: &[MatrixRow] = &[
    // ── adk.session: put ──────────────────────────────────────────────────────
    // Pure, total, value-semantic: returns a new State snapshot — no selection,
    // no accuracy semantics, no effect (ADR-003, RT1).  Exact (C2).
    MatrixRow {
        op: "adk.session::put",
        guarantee: GuaranteeTag::Exact,
        fallibility: Fallibility::Total,
        error_set: "",
        effects: "none",
        explainable: Explainable::NotApplicable,
        llm_output: IsLlmOutput::No,
    },
    // ── adk.session: append_event ─────────────────────────────────────────────
    // Append-only, pure, total: returns a new Session with one more Event (ADR-003).
    // Exact (C2).
    MatrixRow {
        op: "adk.session::append_event",
        guarantee: GuaranteeTag::Exact,
        fallibility: Fallibility::Total,
        error_set: "",
        effects: "none",
        explainable: Explainable::NotApplicable,
        llm_output: IsLlmOutput::No,
    },
    // ── adk.session: State::get ───────────────────────────────────────────────
    // Pure, fallible: returns Option<&Value>; None for missing key — never a silent default (C1/G2).
    // Exact (C2 — no accuracy semantics, just a lookup).  The Option is the Fallible surface:
    // callers must handle the None case; no sentinel / silent default is ever substituted.
    MatrixRow {
        op: "adk.session::State::get",
        guarantee: GuaranteeTag::Exact,
        fallibility: Fallibility::Fallible,
        error_set: "None — missing key (Option; never a silent default)",
        effects: "none",
        explainable: Explainable::NotApplicable,
        llm_output: IsLlmOutput::No,
    },
    // ── adk.tool: Tool::run ───────────────────────────────────────────────────
    // Explicit Result<Out, ToolError>; never silent (C1).
    // Exact for the dispatch itself — the *content*'s tag is the upstream's (meet).
    MatrixRow {
        op: "adk.tool::Tool::run",
        guarantee: GuaranteeTag::Exact,
        fallibility: Fallibility::Fallible,
        error_set: "Err(BadArgs|OutOfDomain|Refused|Upstream)",
        effects: "none",
        explainable: Explainable::Yes,
        llm_output: IsLlmOutput::No,
    },
    // ── adk.model: LlmOutcome::new ────────────────────────────────────────────
    // Constructs a tagged LLM outcome.  Returns Err if tag ∉ {Declared, Empirical}.
    // The *outcome value's tag* is Declared (the safe floor for a new outcome with
    // no trial basis).
    // Fallible: refuses Proven/Exact (the honesty guard — RFC-0023 §6.1).
    MatrixRow {
        op: "adk.model::LlmOutcome::new",
        guarantee: GuaranteeTag::Declared,
        fallibility: Fallibility::Fallible,
        error_set: "Err(ModelError::Decode) — if tag ∉ {Declared, Empirical} (RFC-0023 §6.1)",
        effects: "none",
        explainable: Explainable::Yes,
        llm_output: IsLlmOutput::Yes,
    },
    // ── adk.model: LlmOutcome (Empirical path) ────────────────────────────────
    // An outcome tagged Empirical — valid when the output quality has been trial-validated.
    // Still an LLM output row: tag must not be Proven/Exact (RFC-0023 §6.1).
    MatrixRow {
        op: "adk.model::LlmOutcome[Empirical]",
        guarantee: GuaranteeTag::Empirical,
        fallibility: Fallibility::Fallible,
        error_set: "Err(ModelError::Decode) — if tag ∉ {Declared, Empirical}",
        effects: "none",
        explainable: Explainable::Yes,
        llm_output: IsLlmOutput::Yes,
    },
    // ── adk.model: is_synthetic ───────────────────────────────────────────────
    // Pure, total honesty gate: returns bool — true iff outcome is mock/fixture.
    // Exact (a total predicate; no accuracy semantics).
    MatrixRow {
        op: "adk.model::is_synthetic",
        guarantee: GuaranteeTag::Exact,
        fallibility: Fallibility::Total,
        error_set: "",
        effects: "none",
        explainable: Explainable::NotApplicable,
        llm_output: IsLlmOutput::No,
    },
    // ── adk.model: model_allowed_tags ────────────────────────────────────────
    // Pure, total: returns the constant {Declared, Empirical} allowed set.
    // Exact (a constant lookup; no accuracy semantics).
    MatrixRow {
        op: "adk.model::model_allowed_tags",
        guarantee: GuaranteeTag::Exact,
        fallibility: Fallibility::Total,
        error_set: "",
        effects: "none",
        explainable: Explainable::NotApplicable,
        llm_output: IsLlmOutput::No,
    },
    // ── adk.model: generate (deferred stub) ──────────────────────────────────
    // Deferred (M-381/M-646): returns Err(ModelError::GenerateDeferred) always.
    // Declared: the stub asserts no quality — the most honest possible tag for a
    // "not yet implemented" op (FLAG M-381/M-646).
    MatrixRow {
        op: "adk.model::generate",
        guarantee: GuaranteeTag::Declared,
        fallibility: Fallibility::Fallible,
        error_set: "Err(ModelError::GenerateDeferred) — always (FLAG M-381/M-646); \
                    Err(MissingKey|ModelUnavailable|SpendCapped|Decode) when real",
        effects: "none",
        explainable: Explainable::Yes,
        llm_output: IsLlmOutput::Yes,
    },
    // ── adk.runner: run (deferred stub) ──────────────────────────────────────
    // Deferred (R23-Q1): returns Err(RunError::AgentFailed(reason)) always.
    // Declared: the stub asserts nothing about agent execution quality.
    MatrixRow {
        op: "adk.runner::run",
        guarantee: GuaranteeTag::Declared,
        fallibility: Fallibility::Fallible,
        error_set: "Err(AgentFailed(reason)) — always while R23-Q1 unresolved (FLAG); \
                    Err(BudgetExhausted|Cancelled) when real",
        effects: "none",
        explainable: Explainable::Yes,
        llm_output: IsLlmOutput::No,
    },
];

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::{Explainable, Fallibility, GuaranteeTag, IsLlmOutput, MATRIX};

    // ── Completeness guard ────────────────────────────────────────────────────

    /// Every op named in the spec §4 appears in the matrix exactly once.
    /// Guard: removing or renaming any op from MATRIX makes this fail.
    #[test]
    fn matrix_contains_all_expected_ops() {
        let expected = [
            "adk.session::put",
            "adk.session::append_event",
            "adk.session::State::get",
            "adk.tool::Tool::run",
            "adk.model::LlmOutcome::new",
            "adk.model::LlmOutcome[Empirical]",
            "adk.model::is_synthetic",
            "adk.model::model_allowed_tags",
            "adk.model::generate",
            "adk.runner::run",
        ];
        for name in &expected {
            assert!(
                MATRIX.iter().any(|r| r.op == *name),
                "matrix is missing op {:?} (spec §4 / RFC-0023)",
                name
            );
        }
        assert_eq!(
            MATRIX.len(),
            expected.len(),
            "matrix has unexpected extra or missing rows"
        );
    }

    // ── THE LOAD-BEARING LLM-TAG GUARD (RFC-0023 §6.1) ───────────────────────
    //
    // This test FAILS if any LLM-output row's tag is `Proven` or `Exact`.
    // It is the mechanical enforcement of the ADK phylum's honesty differentiator:
    // "an LLM output is type-forbidden from Proven/Exact" (RFC-0023 §6.1 / VR-5).
    //
    // Removing or weakening this test is a honesty violation (VR-5).

    /// LOAD-BEARING GUARD: every LLM-output row's tag MUST be `Declared` or `Empirical`.
    /// Guard: changing any `IsLlmOutput::Yes` row to `Proven`/`Exact` makes this fail.
    ///
    /// This test is the matrix-level twin of `model.rs::load_bearing_guard_llm_outcome_refuses_proven_and_exact`.
    /// Both must pass; passing only one is a gap.
    #[test]
    fn llm_output_tag_is_declared_or_empirical() {
        let violations: Vec<&str> = MATRIX
            .iter()
            .filter(|r| r.llm_output == IsLlmOutput::Yes)
            .filter(|r| !r.guarantee.is_allowed_for_llm_output())
            .map(|r| r.op)
            .collect();

        assert!(
            violations.is_empty(),
            "HONESTY VIOLATION (RFC-0023 §6.1 / VR-5): the following LLM-output rows \
             carry a forbidden tag (must be Declared or Empirical, never Proven/Exact): {violations:?}"
        );
    }

    /// GUARD: no LLM-output row is tagged `Proven` (a checked theorem that an LLM never has).
    #[test]
    fn no_llm_output_row_is_proven() {
        for row in MATRIX {
            if row.llm_output == IsLlmOutput::Yes {
                assert_ne!(
                    row.guarantee,
                    GuaranteeTag::Proven,
                    "op {:?} is an LLM-output row but claims Proven — \
                     an LLM has no checked theorem basis (RFC-0023 §6.1 / VR-5)",
                    row.op
                );
            }
        }
    }

    /// GUARD: no LLM-output row is tagged `Exact` (an LLM output is never exact).
    #[test]
    fn no_llm_output_row_is_exact() {
        for row in MATRIX {
            if row.llm_output == IsLlmOutput::Yes {
                assert_ne!(
                    row.guarantee,
                    GuaranteeTag::Exact,
                    "op {:?} is an LLM-output row but claims Exact — \
                     an LLM output is never exact (RFC-0023 §6.1 / VR-5)",
                    row.op
                );
            }
        }
    }

    // ── Fallibility consistency ───────────────────────────────────────────────

    /// Fallible ops have a non-empty error set; total ops have an empty one (C1).
    #[test]
    fn fallibility_and_error_set_are_consistent() {
        for row in MATRIX {
            match row.fallibility {
                Fallibility::Total => assert!(
                    row.error_set.is_empty(),
                    "total op {:?} must have an empty error_set (C1)",
                    row.op
                ),
                Fallibility::Fallible => assert!(
                    !row.error_set.is_empty(),
                    "fallible op {:?} must name its error set (C1)",
                    row.op
                ),
            }
        }
    }

    // ── Tool-call rows are explicit ───────────────────────────────────────────

    /// Tool-call rows return explicit `Result` — never total (RFC-0023 §6.2).
    /// Guard: flipping `adk.tool::Tool::run` to `Total` re-introduces silent failure paths.
    #[test]
    fn tool_call_rows_are_fallible_never_total() {
        let tool_ops = ["adk.tool::Tool::run"];
        for op in &tool_ops {
            let row = MATRIX
                .iter()
                .find(|r| r.op == *op)
                .unwrap_or_else(|| panic!("op {:?} missing from matrix", op));
            assert_eq!(
                row.fallibility,
                Fallibility::Fallible,
                "tool-call op {:?} must be Fallible — never total (RFC-0023 §6.2 / C1 / G2)",
                op
            );
        }
    }

    // ── Pure session ops are total + Exact ───────────────────────────────────

    /// The pure session ops (`put`, `append_event`) are `Total` + `Exact` + effect-free.
    /// Guard: making any of these `Fallible` or adding an effect makes this fail.
    #[test]
    fn pure_session_ops_are_total_exact_and_effect_free() {
        let pure_ops = ["adk.session::put", "adk.session::append_event"];
        for op in &pure_ops {
            let row = MATRIX
                .iter()
                .find(|r| r.op == *op)
                .unwrap_or_else(|| panic!("op {:?} missing", op));
            assert_eq!(
                row.guarantee,
                GuaranteeTag::Exact,
                "pure session op {:?} must be Exact (no accuracy semantics — C2)",
                op
            );
            assert_eq!(
                row.fallibility,
                Fallibility::Total,
                "pure session op {:?} must be Total (value-semantic, no failure path)",
                op
            );
            assert_eq!(
                row.effects, "none",
                "pure session op {:?} must declare no effects (pure)",
                op
            );
        }
    }

    // ── Deferred stubs are Declared ───────────────────────────────────────────

    /// Deferred stubs (`generate`, `run`) are tagged `Declared` (the honest floor for
    /// "not yet implemented").
    /// Guard: tagging a deferred stub `Exact` or `Proven` overclaims (VR-5).
    #[test]
    fn deferred_stubs_are_declared() {
        let deferred_ops = ["adk.model::generate", "adk.runner::run"];
        for op in &deferred_ops {
            let row = MATRIX
                .iter()
                .find(|r| r.op == *op)
                .unwrap_or_else(|| panic!("op {:?} missing", op));
            assert_eq!(
                row.guarantee,
                GuaranteeTag::Declared,
                "deferred op {:?} must be Declared (honest floor for unimplemented — VR-5)",
                op
            );
            assert_eq!(
                row.fallibility,
                Fallibility::Fallible,
                "deferred op {:?} must be Fallible (explicit Err — never a fabricated Ok)",
                op
            );
        }
    }

    // ── Explainability ────────────────────────────────────────────────────────

    /// Fallible ops that carry a reason in their `Err` are `Explainable::Yes`.
    /// Guard: removing `Explainable::Yes` from a refusal-carrying op makes this fail.
    #[test]
    fn fallible_ops_carrying_reason_are_explainable() {
        let explain_ops = [
            "adk.tool::Tool::run",
            "adk.model::LlmOutcome::new",
            "adk.model::LlmOutcome[Empirical]",
            "adk.model::generate",
            "adk.runner::run",
        ];
        for op in &explain_ops {
            let row = MATRIX
                .iter()
                .find(|r| r.op == *op)
                .unwrap_or_else(|| panic!("op {:?} missing", op));
            assert_eq!(
                row.explainable,
                Explainable::Yes,
                "op {:?} carries a reason in Err — must be Explainable::Yes (C3)",
                op
            );
        }
    }

    /// Pure ops with no selection/conversion are `NotApplicable` for EXPLAIN.
    #[test]
    fn pure_ops_are_not_applicable_for_explain() {
        let na_ops = [
            "adk.session::put",
            "adk.session::append_event",
            "adk.session::State::get",
            "adk.model::is_synthetic",
            "adk.model::model_allowed_tags",
        ];
        for op in &na_ops {
            let row = MATRIX
                .iter()
                .find(|r| r.op == *op)
                .unwrap_or_else(|| panic!("op {:?} missing", op));
            assert_eq!(
                row.explainable,
                Explainable::NotApplicable,
                "pure op {:?} must be NotApplicable for EXPLAIN (C3 n/a)",
                op
            );
        }
    }

    // ── Mock-outcome honesty guard ────────────────────────────────────────────

    /// `is_synthetic` is `Exact` and effect-free — the primary honesty gate (RFC-0023 §6.5).
    #[test]
    fn is_synthetic_is_exact_and_effect_free() {
        let row = MATRIX
            .iter()
            .find(|r| r.op == "adk.model::is_synthetic")
            .expect("is_synthetic row must be in matrix");
        assert_eq!(
            row.guarantee,
            GuaranteeTag::Exact,
            "is_synthetic must be Exact (a total predicate; RFC-0023 §6.5)"
        );
        assert_eq!(row.effects, "none", "is_synthetic must be effect-free");
        assert_eq!(
            row.fallibility,
            Fallibility::Total,
            "is_synthetic must be Total (never fails — RFC-0023 §6.5)"
        );
    }
}
