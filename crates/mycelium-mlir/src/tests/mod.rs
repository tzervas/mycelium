//! In-crate test modules for `mycelium-mlir` (CLAUDE.md test-layout rule).
//! White-box access via `use crate::…::*`; logic files carry no `#[cfg(test)]` inline code.

mod accel;
mod bitnet;
mod dialect;
mod inject_tests;
mod jit;
mod mode;
// `dialect::native` only compiles under `mlir-dialect`, so its white-box tests are gated to match.
#[cfg(feature = "mlir-dialect")]
mod native;
mod passes;
mod rc_plan_tests;
