//! In-crate test modules for `mycelium-mlir` (CLAUDE.md test-layout rule).
//! White-box access via `use crate::…::*`; logic files carry no `#[cfg(test)]` inline code.

mod dialect;
mod inject_tests;
mod rc_plan_tests;
