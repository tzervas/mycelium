//! Per-operation guarantee matrix for the `adk` phylum (RFC-0016 §4.5 / RFC-0023 §6).
//!
//! TODO(leaf ADK / M-671): encode one row per exported op **as data** (never prose-only) and
//! assert it with guard tests. The load-bearing honesty guard (RFC-0023 §6.1): **every
//! LLM-output row is tagged `Declared` or `Empirical` — never `Proven`/`Exact`** — plus tool
//! failures are explicit and mocks are flagged. Copy the exemplar at
//! `crates/mycelium-std-io/src/guarantee_matrix.rs`.
