//! `mycelium-lsp` — the **minimal toolchain surface** (FR-S5; Foundation §5.8): the invariant
//! linter (M-141), the canonical formatter (M-142), and the LSP feedback facade (M-140) that
//! exposes the four semantic-feedback artifact kinds over one surface (SC-5 channel).
//!
//! This is a *toolchain* crate, deliberately kept out of the small auditable kernel (KC-3): it
//! depends on `mycelium-core`/`-interp`/`-cert` but nothing depends on it.

pub mod lint;

pub use lint::{has_errors, lint, Diagnostic, Severity};
