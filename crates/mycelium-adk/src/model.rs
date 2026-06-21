//! `adk.model` — model layer honesty discipline (RFC-0023 §4.5 + §6.1).
//!
//! The **central honesty deliverable** of the `adk` phylum: an LLM outcome's
//! guarantee tag is **type-forbidden** from `Proven`/`Exact` (RFC-0023 §6.1).
//!
//! ## The honesty rule (VR-5 / RFC-0023 §6.1)
//! > ADK's Python port returns untyped `str`/`dict` — a hallucination and a
//! > theorem are the *same type*.  In Mycelium every model output carries a
//! > guarantee tag, and the substrate **type-forbids** tagging a model output
//! > `Proven`/`Exact` — only `Declared` or `Empirical` are allowed.
//!
//! ## Enforcement (not just prose)
//! - [`model_allowed_tags`] returns the explicit allowed set.
//! - [`LlmOutcome::new`] and [`LlmOutcome::with_tag`] return `Err` if the caller
//!   supplies `Proven` or `Exact` — never a silent coercion.
//! - [`is_synthetic`] is the honesty gate: a mocked/fixture run is never presented
//!   as real model quality (RFC-0023 §6.5 — `llm.rs:5-9,146-152,410-419`).
//! - The guarantee_matrix guard test **fails** if any LLM-output row carries
//!   `Proven`/`Exact`.
//!
//! ## Types
//! - [`ModelError`] — `MissingKey | ModelUnavailable | SpendCapped(String) | Decode(String)`
//! - [`LlmRequest`] — model I/O request shape
//! - [`LlmOutcome`] — tagged model response (tag restricted to `Declared`/`Empirical`)
//! - [`GuaranteeTag`] — the local tag enum (mirrors `GuaranteeStrength`; self-contained)
//!
//! ## Deferred
//! - Real `generate` calls (M-381/M-646 excluded this wave) — returns explicit `Err`.
//! - The `graft` surface (E7-2/M-664) — not yet active.
//!
//! ## Design spec
//! `docs/rfcs/RFC-0023-Agent-Development-Kit-Phylum.md` §4.5, §6.1, §6.5

use std::fmt;

// ── GuaranteeTag ──────────────────────────────────────────────────────────────

/// The honesty lattice tag for an LLM outcome (RFC-0023 §6.1; VR-5).
///
/// Only `Declared` and `Empirical` are allowed for LLM outputs — `Proven`/`Exact`
/// are **forbidden** (an LLM has no checked basis).  The set `{Declared, Empirical}`
/// is the [`model_allowed_tags`] return value and is asserted in the guarantee matrix.
///
/// This is a local projection of `mycelium_core::guarantee::GuaranteeStrength` for
/// self-contained use in this crate (avoiding a dependency chain for pure-data tests).
/// The lattice order is: `Exact ⊐ Proven ⊐ Empirical ⊐ Declared`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuaranteeTag {
    /// No approximation; basis is a complete check.  **FORBIDDEN for LLM outputs.**
    Exact,
    /// A machine-checked theorem establishes the bound.  **FORBIDDEN for LLM outputs.**
    Proven,
    /// The property is established by an empirical corpus (proptest / trial).
    /// Allowed for LLM outputs when the output quality has been trial-validated.
    Empirical,
    /// The property is asserted without a checked basis.  The honest floor for an
    /// LLM output when no measurement exists.  Always the safe default.
    Declared,
}

impl GuaranteeTag {
    /// `true` iff this tag is allowed for an LLM output (RFC-0023 §6.1).
    ///
    /// Only `Declared` and `Empirical` return `true`.  `Exact` and `Proven` always
    /// return `false` — an LLM has no checked basis for either.
    #[must_use]
    pub fn is_allowed_for_llm_output(self) -> bool {
        matches!(self, GuaranteeTag::Declared | GuaranteeTag::Empirical)
    }

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
}

impl fmt::Display for GuaranteeTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The set of guarantee tags allowed for an LLM output (RFC-0023 §6.1; `llm.rs:63-64`).
///
/// Only `{Declared, Empirical}` — never `Proven` or `Exact`.  This is a
/// **load-bearing constant**: the guarantee matrix guard test asserts every
/// LLM-output row's tag ∈ this set.
pub const ALLOWED_LLM_TAGS: [GuaranteeTag; 2] = [GuaranteeTag::Declared, GuaranteeTag::Empirical];

/// Returns the set of tags allowed for an LLM output as a slice.
///
/// The caller should use this to validate a tag before constructing an [`LlmOutcome`].
/// Asserting that `generate`'s result tag ∈ `model_allowed_tags()` is the primary
/// honesty gate (RFC-0023 §6.1).
#[must_use]
pub fn model_allowed_tags() -> &'static [GuaranteeTag] {
    &ALLOWED_LLM_TAGS
}

// ── ModelError ────────────────────────────────────────────────────────────────

/// The explicit error set for model layer failures (C1 / RFC-0023 §4.5).
///
/// Every variant is a named, inspectable failure — never a sentinel or a fabricated
/// answer (RFC-0023 §5 — "a missing key/model is an explicit `Err`, never a fabricated
/// answer"; `README.md:48-49,476`; C1 `RFC-0016:82-84`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelError {
    /// The required API key was missing or empty.
    ///
    /// C1: never silently use a blank key; refuse explicitly.
    MissingKey,
    /// The requested model backend is not available (unknown model id, offline, etc.).
    ///
    /// C1: never fabricate a response; refuse explicitly.
    ModelUnavailable,
    /// The declared USD spend cap would be exceeded by this request.
    ///
    /// `0` carries the budget detail (e.g. `"$0.05 estimated > $0.01 cap"`).
    /// The gate is **best-effort, not a formal bound** (RFC-0023 §5 — "best-effort,
    /// not a formal bound"), so the cost guarantee is `Declared`, never `Proven`.
    SpendCapped(String),
    /// The model's response could not be decoded into a valid `LlmOutcome`.
    ///
    /// `0` carries the decode failure description.  Malformed input is a loud error,
    /// never a silent drop (G2 — `llm.rs:177-180,240-242`).
    Decode(String),

    /// A real `generate` call was attempted but this operation is deferred (FLAG).
    ///
    /// FLAG (M-381/M-646): real LLM calls are excluded from this wave.  The `generate`
    /// stub returns this variant — an explicit `Err`, never a fabricated `Ok`.
    GenerateDeferred,
}

impl fmt::Display for ModelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ModelError::MissingKey => write!(f, "model API key missing"),
            ModelError::ModelUnavailable => write!(f, "model backend unavailable"),
            ModelError::SpendCapped(detail) => write!(f, "spend cap exceeded: {detail}"),
            ModelError::Decode(why) => write!(f, "model response decode failed: {why}"),
            ModelError::GenerateDeferred => write!(
                f,
                "adk.model generate is deferred (M-381/M-646): real LLM calls excluded this wave"
            ),
        }
    }
}

mycelium_std_core::impl_std_error!(ModelError);

// ── LlmRequest ────────────────────────────────────────────────────────────────

/// The model I/O request shape (RFC-0023 §4.5; mirrors `GrokLlmReport` from `llm.rs:243-258`).
///
/// This is pure data — no network call, no effect.  The real call is made by the
/// `generate` stub (deferred, M-381/M-646).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LlmRequest {
    /// The model identifier to invoke (e.g. `"grok-3"`, `"claude-3-5-sonnet"`).
    pub model_id: String,
    /// The system instruction for this invocation.
    pub system: String,
    /// The conversation history / user messages.
    pub messages: Vec<String>,
    /// Maximum tokens to generate (explicit budget; never silent).
    pub max_tokens: Option<u32>,
}

// ── LlmOutcome ────────────────────────────────────────────────────────────────

/// A tagged model response — the load-bearing honesty type (RFC-0023 §4.5, §6.1).
///
/// The tag is **restricted** to `Declared` or `Empirical`; `Proven`/`Exact` are
/// refused by [`LlmOutcome::new`] / [`LlmOutcome::with_tag`].
///
/// An outcome is **synthetic** (mock/fixture) iff `synthetic == true` — a synthetic
/// outcome is **never** presented as real model quality ([`is_synthetic`]).
///
/// # Honesty contract (VR-5 / RFC-0023 §6.1 / §6.5)
/// - `tag` ∈ `{Declared, Empirical}` — always.  The constructors enforce this.
/// - `synthetic == true` ⇒ this outcome is from a mock/fixture run; the runner must
///   flag it as synthetic in any report.
/// - The tag is **never upgraded** between the model call and the caller — it is
///   preserved verbatim (`llm.rs:367-368,507-508`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LlmOutcome {
    /// The model's text output.
    pub text: String,
    /// The guarantee tag.  Always `Declared` or `Empirical` — constructors enforce this.
    tag: GuaranteeTag,
    /// `true` iff this outcome is from a mock/fixture run, not a real model call.
    ///
    /// A synthetic outcome must **never** be reported as real model quality
    /// (RFC-0023 §6.5 — "is_synthetic() is the primary honesty gate").
    synthetic: bool,
    /// The model id that produced this outcome.
    pub model_id: String,
    /// Usage metadata (e.g. prompt tokens, completion tokens) — for audit/budget.
    pub usage: Option<UsageMetadata>,
}

/// Token usage metadata from a model response.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsageMetadata {
    /// Number of tokens in the prompt.
    pub prompt_tokens: u32,
    /// Number of tokens in the completion.
    pub completion_tokens: u32,
}

impl LlmOutcome {
    /// Construct an `LlmOutcome` with the given tag.
    ///
    /// # Errors
    /// Returns `Err(ModelError::Decode(_))` if `tag` is `Proven` or `Exact` —
    /// **never** silently coerces a forbidden tag (VR-5 / RFC-0023 §6.1).
    ///
    /// # Honesty guarantee
    /// The tag is stored verbatim — never upgraded, never rounded.
    pub fn new(
        text: impl Into<String>,
        tag: GuaranteeTag,
        model_id: impl Into<String>,
    ) -> Result<Self, ModelError> {
        if !tag.is_allowed_for_llm_output() {
            return Err(ModelError::Decode(format!(
                "LLM output tag `{tag}` is forbidden — allowed: Declared, Empirical (RFC-0023 §6.1 / VR-5)"
            )));
        }
        Ok(LlmOutcome {
            text: text.into(),
            tag,
            synthetic: false,
            model_id: model_id.into(),
            usage: None,
        })
    }

    /// Construct a **synthetic** (mock/fixture) `LlmOutcome`.
    ///
    /// Synthetic outcomes are always tagged `Declared` (the safest honest floor
    /// for a fixture) and `synthetic == true`.
    ///
    /// A synthetic outcome **must not** be reported as real model quality.
    pub fn synthetic(text: impl Into<String>, model_id: impl Into<String>) -> Self {
        LlmOutcome {
            text: text.into(),
            tag: GuaranteeTag::Declared,
            synthetic: true,
            model_id: model_id.into(),
            usage: None,
        }
    }

    /// Return the guarantee tag of this outcome (read-only; never upgradeable).
    #[must_use]
    pub fn tag(&self) -> GuaranteeTag {
        self.tag
    }

    /// Attempt to set a new guarantee tag on this outcome.
    ///
    /// # Errors
    /// Returns `Err(ModelError::Decode(_))` if `new_tag` is `Proven` or `Exact`
    /// (RFC-0023 §6.1 — the tag may be downgraded but **never upgraded** to a
    /// stronger-than-allowed level; VR-5).
    pub fn with_tag(mut self, new_tag: GuaranteeTag) -> Result<Self, ModelError> {
        if !new_tag.is_allowed_for_llm_output() {
            return Err(ModelError::Decode(format!(
                "cannot set LLM outcome tag to `{new_tag}` — forbidden (allowed: Declared, Empirical; VR-5)"
            )));
        }
        self.tag = new_tag;
        Ok(self)
    }

    /// Attach usage metadata to this outcome (builder method).
    #[must_use]
    pub fn with_usage(mut self, usage: UsageMetadata) -> Self {
        self.usage = Some(usage);
        self
    }
}

/// Returns `true` iff `outcome` is synthetic (mock/fixture, not a real model call).
///
/// A synthetic outcome **must never** be presented as evidence of real model quality
/// (RFC-0023 §6.5 — `llm.rs:5-9,146-152,410-419`).
///
/// # Guarantee: `Exact` (total predicate, no approximation).
#[must_use]
pub fn is_synthetic(outcome: &LlmOutcome) -> bool {
    outcome.synthetic
}

/// A deferred stub for the real `generate` operation.
///
/// Returns `Err(ModelError::GenerateDeferred)` always.  This is **not** a fabricated
/// success — it is an explicit `Err` signalling that real LLM calls are excluded from
/// this wave (M-381/M-646 gate; RFC-0023 §7.4 — "ship Rust-first … with honest tags").
///
/// ## FLAG (M-381/M-646)
/// Real `generate` calls are excluded this wave.  When M-381/M-646 land, this stub
/// is replaced by an actual harness call.  The return type `Result<LlmOutcome, ModelError>`
/// is correct — do not change it when implementing.
///
/// ## FLAG (E7-2 / M-664 — graft surface)
/// The Mycelium target syntax is:
/// ```mycelium
/// graft generate(cap: Substrate{Xai}, req: LlmRequest, budget: UsdBudget)
///     -> Result<LlmOutcome @ Empirical, ModelError>
/// ```
/// The `graft`/`consume` keyword surface and the `Substrate` capability type are not
/// yet active (E7-2/M-664).
pub fn generate(_req: &LlmRequest) -> Result<LlmOutcome, ModelError> {
    Err(ModelError::GenerateDeferred)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::{
        generate, is_synthetic, model_allowed_tags, GuaranteeTag, LlmOutcome, LlmRequest,
        ModelError, ALLOWED_LLM_TAGS,
    };
    use proptest::prelude::*;

    // ── Unit tests ─────────────────────────────────────────────────────────────

    /// `model_allowed_tags()` returns exactly `{Declared, Empirical}` (RFC-0023 §6.1).
    /// Guard: adding `Proven` or `Exact` to the allowed set breaks this.
    #[test]
    fn model_allowed_tags_is_declared_and_empirical_only() {
        let allowed = model_allowed_tags();
        assert_eq!(allowed.len(), 2, "exactly two tags must be allowed");
        assert!(
            allowed.contains(&GuaranteeTag::Declared),
            "Declared must be allowed"
        );
        assert!(
            allowed.contains(&GuaranteeTag::Empirical),
            "Empirical must be allowed"
        );
        assert!(
            !allowed.contains(&GuaranteeTag::Proven),
            "Proven must NOT be allowed for LLM outputs (RFC-0023 §6.1 / VR-5)"
        );
        assert!(
            !allowed.contains(&GuaranteeTag::Exact),
            "Exact must NOT be allowed for LLM outputs (RFC-0023 §6.1 / VR-5)"
        );
    }

    /// `LlmOutcome::new` with `Declared` succeeds (the honest floor for new outputs).
    #[test]
    fn llm_outcome_new_declared_succeeds() {
        let o = LlmOutcome::new("Paris is in France.", GuaranteeTag::Declared, "grok");
        assert!(o.is_ok());
        assert_eq!(o.unwrap().tag(), GuaranteeTag::Declared);
    }

    /// `LlmOutcome::new` with `Empirical` succeeds.
    #[test]
    fn llm_outcome_new_empirical_succeeds() {
        let o = LlmOutcome::new("answer", GuaranteeTag::Empirical, "grok");
        assert!(o.is_ok());
    }

    /// `LlmOutcome::new` with `Proven` is refused — never silent (RFC-0023 §6.1).
    /// Guard: a coercion that returns `Ok` here is a honesty violation.
    #[test]
    fn llm_outcome_new_proven_is_refused_never_silent() {
        let result = LlmOutcome::new("answer", GuaranteeTag::Proven, "grok");
        assert!(
            result.is_err(),
            "Proven tag must be refused for LLM outputs — never coerced silently (VR-5)"
        );
        assert!(matches!(result, Err(ModelError::Decode(_))));
    }

    /// `LlmOutcome::new` with `Exact` is refused — never silent (RFC-0023 §6.1).
    /// Guard: a coercion that returns `Ok` here is a honesty violation.
    #[test]
    fn llm_outcome_new_exact_is_refused_never_silent() {
        let result = LlmOutcome::new("answer", GuaranteeTag::Exact, "grok");
        assert!(
            result.is_err(),
            "Exact tag must be refused for LLM outputs — never coerced silently (VR-5)"
        );
        assert!(matches!(result, Err(ModelError::Decode(_))));
    }

    /// `LlmOutcome::with_tag` refuses `Proven` — the tag cannot be upgraded (VR-5).
    #[test]
    fn with_tag_refuses_proven_upgrade() {
        let o = LlmOutcome::new("x", GuaranteeTag::Declared, "m").unwrap();
        let result = o.with_tag(GuaranteeTag::Proven);
        assert!(
            result.is_err(),
            "upgrading to Proven must be refused (VR-5 — tag is never upgraded)"
        );
    }

    /// `LlmOutcome::with_tag` refuses `Exact` — the tag cannot be upgraded (VR-5).
    #[test]
    fn with_tag_refuses_exact_upgrade() {
        let o = LlmOutcome::new("x", GuaranteeTag::Declared, "m").unwrap();
        let result = o.with_tag(GuaranteeTag::Exact);
        assert!(result.is_err(), "upgrading to Exact must be refused (VR-5)");
    }

    /// `LlmOutcome::with_tag(Empirical)` on a `Declared` outcome is allowed
    /// (Empirical is stronger than Declared but still in the allowed set).
    #[test]
    fn with_tag_allows_empirical_on_declared() {
        let o = LlmOutcome::new("x", GuaranteeTag::Declared, "m")
            .unwrap()
            .with_tag(GuaranteeTag::Empirical)
            .unwrap();
        assert_eq!(o.tag(), GuaranteeTag::Empirical);
    }

    /// A synthetic outcome is always `Declared` and `is_synthetic` returns `true`.
    #[test]
    fn synthetic_outcome_is_declared_and_flagged() {
        let o = LlmOutcome::synthetic("mock answer", "mock-model");
        assert_eq!(o.tag(), GuaranteeTag::Declared);
        assert!(
            is_synthetic(&o),
            "synthetic outcome must be flagged (RFC-0023 §6.5)"
        );
    }

    /// A real outcome (not synthetic) has `is_synthetic == false`.
    #[test]
    fn real_outcome_is_not_synthetic() {
        let o = LlmOutcome::new("real answer", GuaranteeTag::Empirical, "grok").unwrap();
        assert!(
            !is_synthetic(&o),
            "a real outcome must not be marked synthetic"
        );
    }

    /// `generate` returns `Err(GenerateDeferred)` — explicit, never a fabricated success.
    /// Guard: any change that makes `generate` return `Ok` is a honesty violation.
    #[test]
    fn generate_returns_err_generate_deferred_never_fabricates() {
        let req = LlmRequest {
            model_id: "grok".to_owned(),
            system: "be helpful".to_owned(),
            messages: vec!["hello".to_owned()],
            max_tokens: Some(256),
        };
        let result = generate(&req);
        assert!(
            matches!(result, Err(ModelError::GenerateDeferred)),
            "generate must return Err(GenerateDeferred) — real LLM calls excluded this wave (M-381/M-646); got {result:?}"
        );
    }

    /// `ModelError` implements `std::error::Error` (compile-time check).
    #[test]
    fn model_error_is_std_error() {
        let e = ModelError::MissingKey;
        let _: &dyn std::error::Error = &e;
    }

    /// `ModelError` Display is non-empty for every variant (G11 — human-legible).
    #[test]
    fn model_error_display_non_empty_for_every_variant() {
        let variants = [
            ModelError::MissingKey,
            ModelError::ModelUnavailable,
            ModelError::SpendCapped("$1 > $0.10".to_owned()),
            ModelError::Decode("unexpected token".to_owned()),
            ModelError::GenerateDeferred,
        ];
        for v in &variants {
            let s = v.to_string();
            assert!(
                !s.is_empty(),
                "ModelError::{v:?} must have non-empty Display (G11)"
            );
        }
    }

    /// `GuaranteeTag::is_allowed_for_llm_output` matches the `ALLOWED_LLM_TAGS` set.
    #[test]
    fn is_allowed_for_llm_output_matches_allowed_tags_constant() {
        for tag in [
            GuaranteeTag::Exact,
            GuaranteeTag::Proven,
            GuaranteeTag::Empirical,
            GuaranteeTag::Declared,
        ] {
            let allowed_by_method = tag.is_allowed_for_llm_output();
            let allowed_by_const = ALLOWED_LLM_TAGS.contains(&tag);
            assert_eq!(
                allowed_by_method, allowed_by_const,
                "is_allowed_for_llm_output({tag:?}) must agree with ALLOWED_LLM_TAGS"
            );
        }
    }

    // ── THE load-bearing LLM-tag guard test ───────────────────────────────────
    //
    // RFC-0023 §6.1 — "the substrate **type-forbids** tagging a model output
    // `Proven`/`Exact`".  This test is the mechanical guard: it FAILS if the
    // constructors accept `Proven` or `Exact`, ensuring the honesty rule is
    // enforced in code, not just prose.

    /// LOAD-BEARING GUARD: `LlmOutcome` constructors MUST refuse `Proven` and `Exact`.
    /// This test **must fail** if the constructors are weakened to accept forbidden tags.
    /// (RFC-0023 §6.1 / guarantee_matrix.rs `llm_output_tag_is_declared_or_empirical`)
    #[test]
    fn load_bearing_guard_llm_outcome_refuses_proven_and_exact() {
        // Both must be refused — an `Ok` result here is a honesty violation.
        let proven_result = LlmOutcome::new("hallucination", GuaranteeTag::Proven, "any");
        let exact_result = LlmOutcome::new("fact", GuaranteeTag::Exact, "any");

        assert!(
            proven_result.is_err(),
            "HONESTY VIOLATION: LlmOutcome accepted `Proven` tag — \
             an LLM has no checked basis for Proven (RFC-0023 §6.1 / VR-5)"
        );
        assert!(
            exact_result.is_err(),
            "HONESTY VIOLATION: LlmOutcome accepted `Exact` tag — \
             an LLM output is never exact (RFC-0023 §6.1 / VR-5)"
        );

        // `with_tag` must also refuse.
        let o = LlmOutcome::new("x", GuaranteeTag::Declared, "m").unwrap();
        assert!(
            o.clone().with_tag(GuaranteeTag::Proven).is_err(),
            "HONESTY VIOLATION: with_tag accepted Proven upgrade (VR-5)"
        );
        assert!(
            o.with_tag(GuaranteeTag::Exact).is_err(),
            "HONESTY VIOLATION: with_tag accepted Exact upgrade (VR-5)"
        );
    }

    // ── Property tests ────────────────────────────────────────────────────────

    proptest! {
        /// BOUND: For every tag in the lattice, `LlmOutcome::new` returns `Ok` iff the tag
        /// is in `model_allowed_tags()` (Declared or Empirical).
        /// Guard: weakening this to accept Proven/Exact breaks this bound.
        #[test]
        fn prop_llm_outcome_accepts_exactly_the_allowed_tags(
            tag_idx in 0usize..4,
        ) {
            let tags = [
                GuaranteeTag::Exact,
                GuaranteeTag::Proven,
                GuaranteeTag::Empirical,
                GuaranteeTag::Declared,
            ];
            let tag = tags[tag_idx];
            let result = LlmOutcome::new("x", tag, "m");
            let allowed = tag.is_allowed_for_llm_output();
            prop_assert_eq!(
                result.is_ok(),
                allowed,
                "LlmOutcome::new({:?}) must succeed iff tag is in model_allowed_tags()", tag
            );
        }

        /// BOUND: A synthetic outcome is always `Declared` and always `is_synthetic`.
        #[test]
        fn prop_synthetic_outcome_is_always_declared_and_synthetic(text in "[a-z]{1,20}") {
            let o = LlmOutcome::synthetic(text, "mock");
            prop_assert_eq!(o.tag(), GuaranteeTag::Declared,
                "synthetic outcome tag must always be Declared (safe floor)");
            prop_assert!(is_synthetic(&o), "synthetic outcome must be flagged");
        }

        /// BOUND: `LlmOutcome::tag()` always returns the tag it was constructed with
        /// (never upgraded, never transformed).
        #[test]
        fn prop_tag_is_preserved_verbatim(tag_idx in 0usize..2) {
            // Only Declared and Empirical are constructable.
            let allowed_tags = [GuaranteeTag::Declared, GuaranteeTag::Empirical];
            let tag = allowed_tags[tag_idx];
            let o = LlmOutcome::new("x", tag, "m").unwrap();
            prop_assert_eq!(o.tag(), tag, "tag must be preserved verbatim (llm.rs:367-368)");
        }
    }
}
