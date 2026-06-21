//! nodule: `adk.model` — the LLM-harness wiring, per RFC-0023 §4 / §6.1 / §6.5.
//!
//! Build now (pure + testable): `ModelError { MissingKey | ModelUnavailable | SpendCapped |
//! Decode }`, the `LlmRequest` / `LlmOutcome` type shapes, and — critically — the **honesty
//! discipline**: `model_allowed_tags = {Declared, Empirical}`, the outcome's tag preserved
//! **verbatim and never upgraded** to `Proven`/`Exact`, and `is_synthetic(&LlmOutcome) -> bool`
//! so a mock/fixture is never reported as real-model evidence. These are unit/proptest-checked.
//!
//! FLAGGED-gated (never faked): the real `generate(cap, req, budget)` call — real LLM runs are
//! excluded from this wave (M-381/M-646). The deferred path returns an explicit, never-silent `Err`.
