//! In-crate white-box test modules (test layout rule: no tests in logic files; one submodule per
//! source module under test). Extracted as-touched (M-797); new modules land here directly.

mod cert_mode;
mod content;
mod data;
mod id;
#[path = "lib.rs"]
mod lib_root;
mod lower;
mod meta;
/// Shared mode-parametric test harness (M-795; RFC-0034 §13; DN-20). Provides canonical bound
/// fixtures, `for_each_mode`, `ModeScope`, and `assert_mode_scope` for the cross-mode negative
/// pattern. Used by `cert_mode` and `mode_tests`; available to any in-crate test module.
pub(super) mod mode_harness;
mod mode_tests;
mod prim;
mod repr;
mod value;
mod wrapping;
