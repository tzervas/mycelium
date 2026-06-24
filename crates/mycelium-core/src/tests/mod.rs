//! In-crate white-box test modules (test layout rule: no tests in logic files; one submodule per
//! source module under test). Extracted as-touched (M-797); new modules land here directly.

mod cert_mode;
#[path = "lib.rs"]
mod lib_root;
