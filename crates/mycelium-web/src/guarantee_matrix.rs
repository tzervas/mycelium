//! Per-operation guarantee matrix for the `web` phylum (RFC-0016 §4.5 / RFC-0022 §4.5).
//!
//! TODO(leaf WEB / M-670): encode one row per exported op **as data** (never prose-only) —
//! `GuaranteeTag` / `Fallibility` / `Explainable` / `MatrixRow` / `const MATRIX` — and assert it
//! with VR-5 guard tests (no `Proven` without a checked theorem; `io` ops declare the `io` effect;
//! non-finite f64 refused fallibly). Copy the exemplar at
//! `crates/mycelium-std-io/src/guarantee_matrix.rs`.
